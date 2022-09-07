#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused)]

//! Connect Four Game definition
//! This pallet allows accounts to play a game of connect four with each other.

// add the mocks
#[cfg(test)]
mod mock;

// add the tests
#[cfg(test)]
mod tests;

pub use pallet::*;
use sp_std::vec::Vec;
pub type BlockNumber = u64;
use codec::{Decode, Encode};

#[frame_support::pallet]
pub mod pallet {
	use frame_support::debug;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_support::{sp_runtime::app_crypto::sp_core::H256, traits::Randomness};
	use frame_system::pallet_prelude::*;

	// important to use outside structs and consts
	use super::*;

	/// define a pallet struct
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	// disable this so that Vec can be used.
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		// NOTE: Everything added here needs to be added in the `runtime/src/lib.rs` section
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		// points added to the winner's scorecard
		type PointsForWin: Get<u32>;
		// points deducted from the loser's scorecard
		type PointsForLoss: Get<u32>;
		// points added to both players when there's a draw
		type PointsForDraw: Get<u32>;
		/// The pallet doesn't know what the source of randomness is; it can be anything that
		/// implements the trait. When installing this pallet in a runtime, you
		/// must make sure to give it a randomness source that suits its needs.
		type RandomnessSource: Randomness<H256, BlockNumber>;
	}

	/// An enum that indicates the state of a single slot on a ConnectFourBoard.
	#[derive(Encode, Decode, Clone, Default, RuntimeDebug, TypeInfo, PartialEq, Copy)]
	pub enum Coin {
		Player1,
		Player2,
		#[default]
		Empty,
	}

	/// keep track of player in other spaces.
	#[derive(Encode, Decode, Clone, Default, RuntimeDebug, TypeInfo, PartialEq, Copy)]
	pub enum Player {
		#[default]
		One,
		Two,
	}

	#[derive(Encode, Decode, Clone, Default, RuntimeDebug, TypeInfo, PartialEq, Copy)]
	pub enum WinState {
		Player(Player),
		Draw,
		#[default]
		Ongoing,
	}

	/// scorecard for accounts.
	#[derive(Encode, Decode, Clone, Default, RuntimeDebug, TypeInfo)]
	pub struct ScoreCard {
		pub played: u64,
		pub won: u64,
		pub draw: u64,
		pub lost: u64,
		pub ongoing: u64,
		pub points: i64,
	}

	impl ScoreCard {
		fn new() -> Self {
			ScoreCard { played: 0, won: 0, draw: 0, lost: 0, ongoing: 0, points: 0}
		}
	}

	/// A connect four board struct that contains a state matrix.
	/// NOTE: This will inturn be stored in the ConnectFourBoards StorageMap
	#[derive(Encode, Decode, Clone, Default, RuntimeDebug, TypeInfo)]
	pub struct ConnectFourBoardStruct<AccountId> {
		// TODO: perhaps a map can be used instead of a vec?
		pub state: Option<Vec<Vec<Coin>>>,
		/// This is the challenger
		pub player_1: AccountId,
		pub player_2: AccountId,
		pub challenge_accepted: bool,
		id: H256,
		pub active: bool,
		last_played: Option<AccountId>,
		pub win_state: WinState,
	}

	#[derive(Encode, Decode, Clone, Default, RuntimeDebug, TypeInfo)]
	pub enum GameError {
		#[default]
		BoardExistsError,
		NotYourMove,
		NoPlayableMove,
		InvalidColumnForMove,
		ColumnFull,
		BoardNotReady,
		GameEnded,
	}

	impl<AccountId: core::cmp::PartialEq + Clone> ConnectFourBoardStruct<AccountId> {
		/// Constructor for the ConnectFourBoardStruct
		fn new(board_id: H256, player_1: AccountId, player_2: AccountId) -> Self {
			// TODO: randomly create a board ID?
			ConnectFourBoardStruct {
				state: None,
				challenge_accepted: false,
				id: board_id,
				active: false,
				player_1,
				player_2,
				last_played: None,
				win_state: WinState::Ongoing,
			}
		}

		pub fn set_state(&mut self, state: Vec<Vec<Coin>>) {
			self.state = Some(state);
		}

		/// generates the empty board
		fn create_game_board(&mut self) -> Result<(), GameError> {
			match self.state {
				None => {
					let rows = 6;
					let columns = 7;
					let mut state = Vec::with_capacity(rows);
					for _ in 0..rows {
						let mut row = Vec::with_capacity(columns);
						for _ in 0..columns {
							row.push(Coin::Empty);
						}
						state.push(row);
					}
					self.state = Some(state);
					self.active = true;
					Ok(())
				},
				Some(_) => Err(GameError::BoardExistsError),
			}
		}

		/// Gets a winner for a board, if any.
		pub fn get_winner(&mut self) -> WinState {
			match self.win_state {
				WinState::Ongoing => (),
				WinState::Player(player) => return WinState::Player(player),
				WinState::Draw => return WinState::Draw,
			};
			// unwrap the state, which is an option
			let state = self.state.as_ref().unwrap();
			// check horizontally to find 4 consecutive 1s or 2s
			for j in 0..6 {
				for i in 3..7 {
					let current: Coin = state[j][i];
					if current != Coin::Empty {
						let mut won = true;
						for val in &state[j][(i - 3)..(i + 1)] {
							if *val != current {
								won = false;
								break;
							}
						}
						if won {
							log::info!("Horizontal winning condition: j: {j}. Winner={current:?}");
							if current == Coin::Player1 {
								self.active = false;
								self.state = None;
								self.win_state = WinState::Player(Player::One);
								return self.win_state;
							} else if current == Coin::Player2 {
								self.active = false;
								self.state = None;
								self.win_state = WinState::Player(Player::Two);
								return self.win_state;
							};
						}
					};
				}
			}
			// check vertically to find 4 consecutive 1s or 2s
			for i in 0..7 {
				for j in 3..6 {
					let current = state[j][i];
					let mut won = true;
					if current != Coin::Empty {
						for k in (j - 3)..(j + 1) {
							let val = &state[k][i];
							if *val != current {
								won = false;
								break;
							}
						}
						if won {
							log::info!("Vertical winning condition: j: {j}. Winner={current:?}");
							if current == Coin::Player1 {
								self.active = false;
								self.state = None;
								self.win_state = WinState::Player(Player::One);
								return self.win_state;
							} else if current == Coin::Player2 {
								self.active = false;
								self.state = None;
								self.win_state = WinState::Player(Player::Two);
								return self.win_state;
							};
						}
					};
				}
			}
			// check diagonally to find 4 consecutive 1s or 2s
			for i in 0..4 {
				for j in 0..3 {
					// check 1 set of diagonals
					if (state[j][i] == state[j + 1][i + 1])
						&& (state[j][i] == state[j + 2][i + 2])
						&& (state[j][i] == state[j + 3][i + 3])
						&& (state[j][i] != Coin::Empty)
					{
						let current = state[j][i];
						log::info!("Diagonal-1 winning condition: j: {j}. Winner={current:?}");
						if current == Coin::Player1 {
							self.active = false;
							self.state = None;
							self.win_state = WinState::Player(Player::One);
							return self.win_state;
						} else if current == Coin::Player2 {
							self.active = false;
							self.state = None;
							self.win_state = WinState::Player(Player::Two);
							return self.win_state;
						};
					} else if (state[j + 3][i] == state[j + 2][i + 1])
						&& (state[j + 3][i] == state[j + 1][i + 2])
						&& (state[j + 3][i] == state[j][i + 3])
						&& (state[j + 3][i] != Coin::Empty)
					{
						let current = state[j + 3][i];
						log::info!("Diagonal-2 winning condition: j: {j}. Winner={current:?}");
						if current == Coin::Player1 {
							self.active = false;
							self.state = None;
							self.win_state = WinState::Player(Player::One);
							return self.win_state;
						} else if current == Coin::Player2 {
							self.active = false;
							self.state = None;
							self.win_state = WinState::Player(Player::Two);
							return self.win_state;
						};
					};
				}
			}
			// no winner
			if self.is_playable() {
				WinState::Ongoing
			} else {
				self.win_state = WinState::Draw;
				self.active = false;
				self.state = None;
				WinState::Draw
			}
		}
		/// checks if the game has been won.
		pub fn has_winner(&mut self) -> bool {
			let win_state = self.get_winner();
			match win_state {
				WinState::Player(_) => true,
				// NOTE: does a DRAW have a winner?
				_ => false,
			}
		}

		/// checks if the board is playable, i.e., has empty slots anywhere.
		fn is_playable(&self) -> bool {
			let state = match &self.state {
				Some(s) => s,
				None => return false,
			};
			for row in state {
				for column in row {
					if column == &Coin::Empty {
						return true;
					}
				}
			}
			return false;
		}

		/// play a move
		fn play(&mut self, player: AccountId, column: usize) -> Result<(), GameError> {
			// check if the state exists.
			match self.state {
				Some(_) => (),
				None => return Err(GameError::BoardNotReady),
			};
			// check if the game has a winner
			if self.has_winner() {
				return Err(GameError::GameEnded);
			} else if !self.is_playable() {
				// Is this the right place for this?
				self.state = None;
				return Err(GameError::GameEnded);
			}
			// check if the game is unplayable (i.e., no empty slots anywhere.)
			// check if it's the player's turn
			let not_your_move = match &self.last_played {
				Some(last_played) => player == *last_played,
				None => player != self.player_1,
			};
			if not_your_move {
				return Err(GameError::NotYourMove);
			};
			// board doesn't have more than 7 columns
			if column >= 7 {
				return Err(GameError::InvalidColumnForMove);
			};
			let mut has_empty_slot = false;
			let mut empty_slot = 0;
			match &self.state {
				None => Err(GameError::BoardNotReady),
				Some(state) => {
					let mut new_state = state.clone();
					for row in 0..6 {
						if new_state[row][column as usize] == Coin::Empty {
							has_empty_slot = true;
							empty_slot = row;
							break;
						}
					}
					if has_empty_slot {
						if self.player_1 == player {
							new_state[empty_slot][column] = Coin::Player1;
						} else {
							new_state[empty_slot][column] = Coin::Player2;
						}
						self.state = Some(new_state);
						self.last_played = Some(player);
						_ = self.get_winner();

						Ok(())
					} else {
						Err(GameError::ColumnFull)
					}
				},
			}
		}
	}

	// Default value for Nonce
	#[pallet::type_value]
	pub fn NonceDefault<T: Config>() -> u64 {
		0
	}

	/// Nonce storage so that random hashes can be calculated.
	#[pallet::storage]
	pub type Nonce<T: Config> = StorageValue<_, u64, ValueQuery, NonceDefault<T>>;

	/// StorageValue to list all boards
	#[pallet::storage]
	#[pallet::getter(fn get_all_board_ids)]
	pub(super) type ConnectFourBoards<T: Config> = StorageValue<_, Vec<H256>, ValueQuery>;

	/// StorageMap to keep a board per user
	#[pallet::storage]
	#[pallet::getter(fn get_connect_four_board_by_id)]
	pub(super) type ConnectFourBoardById<T: Config> =
		StorageMap<_, Blake2_128Concat, H256, ConnectFourBoardStruct<T::AccountId>, OptionQuery>;

	/// StorageMap to relate account IDs to the challenges
	#[pallet::storage]
	#[pallet::getter(fn get_challenge)]
	pub(super) type Challenges<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		// Recipient
		T::AccountId,
		Blake2_128Concat,
		// challenger
		T::AccountId,
		// Board ID
		H256,
		ValueQuery,
	>;

	/// StorageMap for scorecards
	#[pallet::storage]
	#[pallet::getter(fn get_scorecard)]
	pub(super) type AccountScoreCard<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, ScoreCard, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Challenge has been received
		ChallengeReceived(T::AccountId, T::AccountId),
		/// challenge has been accepted.
		ChallengeAccepted(T::AccountId, T::AccountId),
		/// challenge denied
		ChallengeDenied(T::AccountId, T::AccountId),
		/// A new Game got created.
		GameCreated(H256),
		/// Game ended
		GameEnded(H256),
		/// A user has played a move
		MoveMade(T::AccountId, T::AccountId),
		/// A user has won a game,
		GameWon(T::AccountId, H256),
		/// a game was drawn.
		GameDrawn(H256),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// A challenge already exists between these two accounts.
		ChallengeExists,
		/// A game already exists between these two accounts.
		ActiveGameExists,
		/// Cannot place a token on this column, the column is either full,
		/// or does not exist.
		InvalidColumnForMove,
		/// You are not a participant in this game.
		NotYourBoard,
		/// This game ID is invalid.
		GameDoesNotExist,
		/// There are no more moves possible on this board.
		NoMovesRemaining,
		/// This is not your move. Ask your opponent to make their move.
		NotYourMove,
		/// You cannot start a game against yourself.
		CannotPlayYourself,
		/// You cannot accept your own challenge.
		CannotAcceptYourOwnChallenge,
		/// The current board state is invalid.
		BoardInvalid,
		/// there is no such challenge
		ChallengeDoesNotExist,
		/// Game has ended
		BoardExistsError,
		BoardNotReady,
		ColumnFull,
		GameEnded,
		NoPlayableMove,
		UnknownError,
		ChallengeNotYetAccepted,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Challenge an account to a new game.
		#[pallet::weight(10_000)]
		pub fn challenge_player(origin: OriginFor<T>, opponent: T::AccountId) -> DispatchResult {
			let challenger = ensure_signed(origin)?;
			// Check if the user is trying to challenge themselves to a game.
			ensure!(challenger != opponent, <Error<T>>::CannotPlayYourself);
			// check if a challenge exists already, made by either party.
			let challenge_received = <Challenges<T>>::contains_key(&opponent, &challenger);
			log::info!(
				"Challenge already received={} from {:?} to {:?}",
				challenge_received,
				opponent,
				challenger
			);
			let challenge_sent = <Challenges<T>>::contains_key(&challenger, &opponent);
			log::info!(
				"Challenge already sent={} from {:?} to {:?}",
				challenge_sent,
				challenger,
				opponent
			);
			let challenge_exists = challenge_received || challenge_sent;
			log::info!("Challenge exists: {}", challenge_exists);
			ensure!(!challenge_exists, <Error<T>>::ChallengeExists);
			// no challenge exists between these two.
			// Create new board
			let random_seed = T::RandomnessSource::random_seed();
			// Using a subject is recommended to prevent accidental re-use of the seed
			// (This does not add security or entropy)
			let subject = Self::encode_and_update_nonce();
			let (board_id, block_number) = T::RandomnessSource::random(&subject);

			let board = ConnectFourBoardStruct::<T::AccountId>::new(
				board_id,
				challenger.clone(),
				opponent.clone(),
			);
			// insert the board in the map that has board_id x board
			<ConnectFourBoardById<T>>::insert(&board_id, board);
			// insert this board into the global list of boards;
			let mut boards = <ConnectFourBoards<T>>::get();
			boards.push(board_id);
			<ConnectFourBoards<T>>::set(boards);
			// insert the board_id in the map that has recipient x challenger board_id
			<Challenges<T>>::insert(&opponent, &challenger, board_id);
			// insert the board_id in the map that has challenger x recipient board_id
			<Challenges<T>>::insert(&challenger, &opponent, board_id);
			// send an event saying challenge made
			Self::deposit_event(Event::ChallengeReceived(opponent, challenger));
			Self::deposit_event(Event::GameCreated(board_id));
			Ok(())
		}

		/// Accept a challenge
		#[pallet::weight(10_000)]
		pub fn accept_challenge(origin: OriginFor<T>, challenger: T::AccountId) -> DispatchResult {
			let challenged = ensure_signed(origin)?;
			// cannot accept challenges made by self
			ensure!(challenged != challenger, <Error<T>>::CannotPlayYourself);
			let challenge_exists = <Challenges<T>>::contains_key(&challenged, &challenger);
			ensure!(challenge_exists, <Error<T>>::ChallengeDoesNotExist);
			let board_id = Self::get_challenge(&challenged, &challenger);
			let mut board = Self::get_connect_four_board_by_id(&board_id).unwrap();
			// check whether the player is trying to accept their own challenge.
			// this is doable because player_1 is set to the challenger.
			let own_challenge = board.player_1 == challenged;
			ensure!(!own_challenge, <Error<T>>::CannotAcceptYourOwnChallenge);
			let challenge_already_accepted = board.challenge_accepted;
			ensure!(!challenge_already_accepted, <Error<T>>::ActiveGameExists);
			// TODO: check if the board is completed, if so, a new challenge needs to be issued.
			// Ideally, I should invalidate the map for these address on game completion.
			let game_ongoing = !board.active;
			// let user know if there's already an active game.
			// there's no existing challenge so this can be accepted.
			board.challenge_accepted = true;
			board.create_game_board();
			let mut score_card_1 = Self::get_scorecard(&board.player_1);
			let mut score_card_2 = Self::get_scorecard(&board.player_2);
			score_card_1.played += 1;
			score_card_2.played += 1;
			<AccountScoreCard<T>>::set(&board.player_1, score_card_1);
			<AccountScoreCard<T>>::set(&board.player_2, score_card_2);
			<ConnectFourBoardById<T>>::insert(&board_id, board);
			Self::deposit_event(Event::ChallengeAccepted(challenged, challenger));
			Ok(())
		}

		/// play a move
		#[pallet::weight(10_000)]
		pub fn play(
			origin: OriginFor<T>,
			other_player: T::AccountId,
			column: u32,
		) -> DispatchResult {
			let player = ensure_signed(origin)?;
			// check if such a game exists
			let game_exists = <Challenges<T>>::contains_key(&player, &other_player);
			log::info!("Game between {:?} and {:?} exists={}", player, other_player, game_exists);
			ensure!(game_exists, <Error<T>>::GameDoesNotExist);
			// get the board ID
			let board_id = Self::get_challenge(&player, &other_player);
			// check if the board is active
			let mut board = Self::get_connect_four_board_by_id(&board_id).unwrap();
			ensure!(board.challenge_accepted, <Error<T>>::ChallengeNotYetAccepted);
			ensure!(board.active, <Error<T>>::GameEnded);
			match board.play(player.clone(), column as usize) {
				Err(e) => {
					// returning an error manually is complicated, lets use `ensure!`
					let failure = false;
					match e {
						GameError::BoardExistsError => {
							ensure!(failure, <Error<T>>::BoardExistsError)
						},
						GameError::BoardNotReady => ensure!(failure, <Error<T>>::BoardNotReady),
						GameError::ColumnFull => ensure!(failure, <Error<T>>::ColumnFull),
						GameError::InvalidColumnForMove => {
							ensure!(failure, <Error<T>>::InvalidColumnForMove)
						},
						GameError::NoPlayableMove => ensure!(failure, <Error<T>>::NoPlayableMove),
						GameError::NotYourMove => ensure!(failure, <Error<T>>::NotYourMove),
						GameError::GameEnded => ensure!(failure, <Error<T>>::GameEnded),
					}
					ensure!(failure, <Error<T>>::UnknownError)
				},
				Ok(_) => {
					// need to check if someone has won, or if the game is unplayable.
					match board.get_winner() {
						WinState::Player(winner) => {
							<ConnectFourBoardById<T>>::insert(&board_id, board.clone());
							// remove board from users' storagedoublemap
							<Challenges<T>>::remove(&player, &other_player);
							<Challenges<T>>::remove(&other_player, &player);
							// emit event about the move
							Self::deposit_event(Event::MoveMade(
								player.clone(),
								other_player.clone(),
							));
							// get the scorecards.
							let mut score_card_1 = <AccountScoreCard<T>>::get(&board.player_1);
							let mut score_card_2 = <AccountScoreCard<T>>::get(&board.player_2);
							// emit that a user has won
							match winner {
								Player::One => {
									score_card_1.won = score_card_1.won + 1;
									score_card_2.lost = score_card_2.lost + 1;
									score_card_1.points += T::PointsForWin::get() as i64;
									score_card_2.points -= T::PointsForLoss::get() as i64;
									Self::deposit_event(Event::GameWon(
										board.player_1.clone(),
										board_id,
									));
								},
								Player::Two => {
									score_card_1.lost = score_card_1.lost + 1;
									score_card_2.won = score_card_2.won + 1;
									score_card_1.points -= T::PointsForLoss::get() as i64;
									score_card_2.points += T::PointsForWin::get() as i64;
									// update the scorecards.
									let score_card_1 = <AccountScoreCard<T>>::get(board.player_1.clone());
									let score_card_2 = <AccountScoreCard<T>>::get(board.player_2.clone());
									Self::deposit_event(Event::GameWon(
										board.player_2.clone(),
										board_id,
									));
								},
							};
							// update the score cards
							<AccountScoreCard<T>>::set(&board.player_1, score_card_1);
							<AccountScoreCard<T>>::set(&board.player_2, score_card_2);
							// emit that a game has ended.
							Self::deposit_event(Event::GameEnded(board_id));
						},
						WinState::Draw => {},
						WinState::Ongoing => (),
					}

					if !board.is_playable() {
						// game is not playable.
						// it's a draw
						board.active = false;
						let mut score_card_1 = <AccountScoreCard<T>>::get(&board.player_1);
						score_card_1.draw += 1;
						score_card_1.points += T::PointsForDraw::get() as i64;
						let mut score_card_2 = <AccountScoreCard<T>>::get(&board.player_2);
						score_card_2.draw += 1;
						score_card_2.points += T::PointsForDraw::get() as i64;
						<ConnectFourBoardById<T>>::insert(&board_id, board);
						// remove board from users' storagedoublemap
						<Challenges<T>>::remove(&player, &other_player);
						<Challenges<T>>::remove(&other_player, &player);
						Self::deposit_event(Event::MoveMade(player, other_player));
						Self::deposit_event(Event::GameEnded(board_id));
						Self::deposit_event(Event::GameDrawn(board_id));
					} else {
						<ConnectFourBoardById<T>>::insert(&board_id, board);
						Self::deposit_event(Event::MoveMade(player, other_player));
					}
					return Ok(());
				},
			};
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Reads the nonce from storage, increments the stored nonce, and returns
	/// the encoded nonce to the caller.
	fn encode_and_update_nonce() -> Vec<u8> {
		let nonce = Nonce::<T>::get();
		Nonce::<T>::put(nonce.wrapping_add(1));
		nonce.encode()
	}
}
