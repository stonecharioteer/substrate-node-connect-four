//! tests for connect-four
use crate::{mock::*, Error};
use crate::{Challenges, Coin, ConnectFourBoardStruct, WinState, Player, AccountScoreCard, ConnectFourBoardById};
use frame_support::{assert_noop, assert_ok};

/// An account can challege another account.
#[test]
fn can_challenge_a_player() {
	new_test_ext().execute_with(|| {
		// challenge account 2 from account 1
		assert_ok!(ConnectFour::challenge_player(Origin::signed(1), 2));
		// check if there are challenges between these two, and that the board ID exists.
		let board_id_1 = ConnectFour::get_challenge(1, 2);
		let board_id_2 = ConnectFour::get_challenge(2, 1);
		assert_eq!(board_id_1, board_id_2);
		// there should only be on board
		let boards = ConnectFour::get_all_board_ids();
		assert_eq!(boards.len(), 1);
		// this board should be queriable from the ID
		let board = ConnectFour::get_connect_four_board_by_id(board_id_1).unwrap();
		assert_eq!(board.player_1, 1);
		assert_eq!(board.player_2, 2);
		assert_eq!(board.active, false);
		assert_eq!(board.challenge_accepted, false);
	})
}

/// An account cannot challenge itself
#[test]
fn cannot_challenge_self() {
	new_test_ext().execute_with(|| {
		assert_noop!(ConnectFour::challenge_player(Origin::signed(1), 1), Error::<Test>::CannotPlayYourself);
	})
}

/// An account cannot challenge another when a challenge exists
#[test]
fn cannot_challenge_a_player_twice() {
	new_test_ext().execute_with(|| {
		assert_ok!(ConnectFour::challenge_player(Origin::signed(1), 2));
		assert_noop!(ConnectFour::challenge_player(Origin::signed(1), 2), Error::<Test>::ChallengeExists);
	})
}

/// An account cannot challenge someone who has an active challenge for them.
#[test]
fn cannot_challenge_a_challenger() {
	new_test_ext().execute_with(|| {
		assert_ok!(ConnectFour::challenge_player(Origin::signed(1), 2));
		assert_noop!(ConnectFour::challenge_player(Origin::signed(2), 1), Error::<Test>::ChallengeExists);
	})
}

/// An account can accept a challenge.
#[test]
fn can_accept_a_challenge() {
	new_test_ext().execute_with(|| {
		assert_ok!(ConnectFour::challenge_player(Origin::signed(1), 2));
		assert_ok!(ConnectFour::accept_challenge(Origin::signed(2), 1));
		let score_card_1 = <AccountScoreCard<Test>>::get(1);
		assert_eq!(score_card_1.played, 1);
		let score_card_2 = <AccountScoreCard<Test>>::get(2);
		assert_eq!(score_card_2.played, 1);
	})
}

/// An account cannot accept a challenge it made.
#[test]
fn cannot_accept_own_challenge() {
	new_test_ext().execute_with(|| {
		assert_ok!(ConnectFour::challenge_player(Origin::signed(1), 2));
		assert_noop!(ConnectFour::accept_challenge(Origin::signed(1), 2), Error::<Test>::CannotAcceptYourOwnChallenge);
	})
}

/// An account cannot accept a challenge that doesn't exist
#[test]
fn cannot_accept_non_existent_challenge() {
	new_test_ext().execute_with(|| {
		assert_noop!(ConnectFour::accept_challenge(Origin::signed(1), 2), Error::<Test>::ChallengeDoesNotExist);
	})
}

/// An account cannot challenge itself
#[test]
fn cannot_accept_challenge_to_self() {
	new_test_ext().execute_with(|| {
		assert_noop!(ConnectFour::challenge_player(Origin::signed(1), 1), Error::<Test>::CannotPlayYourself);
	})
}

/// An account cannot accept challenge twice
#[test]
fn cannot_accept_challenge_twice() {
	new_test_ext().execute_with(|| {
		assert_ok!(ConnectFour::challenge_player(Origin::signed(1), 2));
		assert_ok!(ConnectFour::accept_challenge(Origin::signed(2), 1));
		assert_noop!(ConnectFour::accept_challenge(Origin::signed(2), 1), Error::<Test>::ActiveGameExists);
	})
}

/// An account cannot make the first move if it's not the challenger
#[test]
fn cannot_make_first_move_if_not_challenger() {
	new_test_ext().execute_with(|| {
		assert_ok!(ConnectFour::challenge_player(Origin::signed(1), 2));
		assert_ok!(ConnectFour::accept_challenge(Origin::signed(2), 1));
		assert_noop!(ConnectFour::play(Origin::signed(2), 1, 0), Error::<Test>::NotYourMove);
	})
}

/// An account can make the first move if it was the challenger
#[test]
fn can_make_first_move_if_challenger() {
	new_test_ext().execute_with(|| {
		assert_ok!(ConnectFour::challenge_player(Origin::signed(1), 2));
		assert_ok!(ConnectFour::accept_challenge(Origin::signed(2), 1));
		assert_ok!(ConnectFour::play(Origin::signed(1), 2, 0));
	})
}

/// An account cannot make a consecutive move
#[test]
fn cannot_make_consecutive_moves() {
	new_test_ext().execute_with(|| {
		assert_ok!(ConnectFour::challenge_player(Origin::signed(1), 2));
		assert_ok!(ConnectFour::accept_challenge(Origin::signed(2), 1));
		assert_ok!(ConnectFour::play(Origin::signed(1), 2, 0));
		assert_noop!(ConnectFour::play(Origin::signed(1), 2, 0), Error::<Test>::NotYourMove);
	})
}

/// An account can make a move after other player has moved
#[test]
fn can_move_after_other_player_moves() {
	new_test_ext().execute_with(|| {
		assert_ok!(ConnectFour::challenge_player(Origin::signed(1), 2));
		assert_ok!(ConnectFour::accept_challenge(Origin::signed(2), 1));
		assert_ok!(ConnectFour::play(Origin::signed(1), 2, 0));
		assert_ok!(ConnectFour::play(Origin::signed(2), 1, 0));
	})
}

/// An account cannot place a move on a column that is full (there will be 6 rows)
#[test]
fn cannot_place_coin_on_full_column() {
	new_test_ext().execute_with(|| {
		assert_ok!(ConnectFour::challenge_player(Origin::signed(1), 2));
		assert_ok!(ConnectFour::accept_challenge(Origin::signed(2), 1));
		assert_ok!(ConnectFour::play(Origin::signed(1), 2, 0));
		assert_ok!(ConnectFour::play(Origin::signed(2), 1, 0));
		assert_ok!(ConnectFour::play(Origin::signed(1), 2, 0));
		assert_ok!(ConnectFour::play(Origin::signed(2), 1, 0));
		assert_ok!(ConnectFour::play(Origin::signed(1), 2, 0));
		assert_ok!(ConnectFour::play(Origin::signed(2), 1, 0));
		assert_noop!(ConnectFour::play(Origin::signed(1), 2, 0), Error::<Test>::ColumnFull);
	})
}

/// An account can win a simple game.
#[test]
fn can_win_simple_game() {
	new_test_ext().execute_with(|| {
		assert_ok!(ConnectFour::challenge_player(Origin::signed(1), 2));
		assert_ok!(ConnectFour::accept_challenge(Origin::signed(2), 1));
		// get board_id before starting
		let board_id = ConnectFour::get_challenge(1, 2);
		assert_ok!(ConnectFour::play(Origin::signed(1), 2, 0));
		assert_ok!(ConnectFour::play(Origin::signed(2), 1, 1));
		assert_ok!(ConnectFour::play(Origin::signed(1), 2, 0));
		assert_ok!(ConnectFour::play(Origin::signed(2), 1, 1));
		assert_ok!(ConnectFour::play(Origin::signed(1), 2, 0));
		assert_ok!(ConnectFour::play(Origin::signed(2), 1, 1));
		assert_ok!(ConnectFour::play(Origin::signed(1), 2, 0));
		let mut board = ConnectFour::get_connect_four_board_by_id(board_id).unwrap();
		assert_eq!(board.active, false, "The game should no longer be active");
		assert_eq!(board.win_state, WinState::Player(Player::One), "Player 1 should be the winner.");
		assert_eq!(board.state, None, "The game state should be purged and not kept on chain.");
		// ensure that the board is nolonger mapped as an active challenge
		let board_exists = <Challenges<Test>>::contains_key(1, 2);
		assert_eq!(board_exists, false, "The challenge should be unmapped between the two players.");
		// get the score cards for both players
		let score_card_1 = <AccountScoreCard<Test>>::get(1);
		assert_eq!(score_card_1.played, 1);
		assert_eq!(score_card_1.won, 1);
		assert_eq!(score_card_1.lost, 0);
		assert_eq!(score_card_1.draw, 0);
		assert_eq!(score_card_1.points, PointsForWin::get() as i64);
		let score_card_2 = <AccountScoreCard<Test>>::get(2);
		assert_eq!(score_card_2.played, 1);
		assert_eq!(score_card_2.won, 0);
		assert_eq!(score_card_2.lost, 1);
		assert_eq!(score_card_2.draw, 0);
		assert_eq!(score_card_2.points, -1 * PointsForLoss::get() as i64);
	})
}

/// an account cannot play on an ended game.
#[test]
fn cannot_play_ended_game() {
	new_test_ext().execute_with(|| {
		assert_ok!(ConnectFour::challenge_player(Origin::signed(1), 2));
		assert_ok!(ConnectFour::accept_challenge(Origin::signed(2), 1));
		assert_ok!(ConnectFour::play(Origin::signed(1), 2, 0));
		assert_ok!(ConnectFour::play(Origin::signed(2), 1, 1));
		assert_ok!(ConnectFour::play(Origin::signed(1), 2, 0));
		assert_ok!(ConnectFour::play(Origin::signed(2), 1, 1));
		assert_ok!(ConnectFour::play(Origin::signed(1), 2, 0));
		assert_ok!(ConnectFour::play(Origin::signed(2), 1, 1));
		assert_ok!(ConnectFour::play(Origin::signed(1), 2, 0));
		assert_noop!(ConnectFour::play(Origin::signed(2), 1,0), Error::<Test>::GameDoesNotExist);
	})
}

/// checks draw status and points
#[test]
fn can_draw_a_game() {
	new_test_ext().execute_with(|| {
		assert_ok!(ConnectFour::challenge_player(Origin::signed(1), 2));
		assert_ok!(ConnectFour::accept_challenge(Origin::signed(2), 1));
		// get board_id before starting
		let board_id = ConnectFour::get_challenge(1, 2);
		let mut board = ConnectFour::get_connect_four_board_by_id(board_id).unwrap();
		/// set the board state to a definite draw.
		let draw_state = vec![
			vec![Coin::Player1, Coin::Player2, Coin::Player1, Coin::Player2, Coin::Player1, Coin::Player2, Coin::Player1],
			vec![Coin::Player2, Coin::Player1, Coin::Player1, Coin::Player2, Coin::Player2, Coin::Player2, Coin::Player1],
			vec![Coin::Player1, Coin::Player2, Coin::Player2, Coin::Player1, Coin::Player1, Coin::Player2, Coin::Player2],
			vec![Coin::Player2, Coin::Player1, Coin::Player2, Coin::Player1, Coin::Player2, Coin::Player1, Coin::Player1],
			vec![Coin::Player2, Coin::Player1, Coin::Player1, Coin::Player1, Coin::Player2, Coin::Player1, Coin::Player1],
			vec![Coin::Player1, Coin::Player2, Coin::Player1, Coin::Player2, Coin::Player1, Coin::Player2, Coin::Player1],
		];
		board.set_state(draw_state);
		board.active = false;
		let winstate = board.get_winner();
		assert_eq!(winstate, WinState::Draw);
	})
}
/// check several possible win-states
#[test]
fn check_win_states() {
	new_test_ext().execute_with(|| {
		assert_ok!(ConnectFour::challenge_player(Origin::signed(1), 2));
		assert_ok!(ConnectFour::accept_challenge(Origin::signed(2), 1));
		// get board_id before starting
		let board_id = ConnectFour::get_challenge(1, 2);
		let mut board = ConnectFour::get_connect_four_board_by_id(board_id).unwrap();
		let winning_states = [
			vec![
				vec![Coin::Player1, Coin::Player1, Coin::Player1, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
			],
			vec![
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Player2, Coin::Player2, Coin::Player2, Coin::Player2, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
			],
			vec![
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Player1, Coin::Player1, Coin::Player1, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
			],
			vec![
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player2, Coin::Player2, Coin::Player2, Coin::Player2],
			],
			vec![
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Player1, Coin::Player1, Coin::Player1, Coin::Player1, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
			],
			vec![
				vec![Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
			],
			vec![
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player2, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player2, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player2, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player2, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
			],
			vec![
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Player2, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Player2, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Player2, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Player2, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
			],
			vec![
				// diagonal-Coin::Player1
				vec![Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
			],
			vec![
				// diagonal-Coin::Player1
				vec![Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
			],
			vec![
				// diagonal-Coin::Player1
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
			],
			vec![
				// diagonal-Coin::Player1
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty],
			],
			vec![
				// diagonal-Coin::Player1
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
			],
			vec![
				// diagonal-Coin::Player2
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Player1, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Player2, Coin::Player1, Coin::Player2, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Player1, Coin::Player2, Coin::Player2, Coin::Player2, Coin::Player1, Coin::Empty, Coin::Empty],
			],
			vec![
				// diagonal-Coin::Player2
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Player2],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Player2, Coin::Player1],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Player2, Coin::Player1, Coin::Player2],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Player2, Coin::Player2, Coin::Player1],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player2, Coin::Player1, Coin::Player2, Coin::Player1],
			],
			vec![
				// diagonal-Coin::Player2
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
			],
			vec![
				// diagonal-Coin::Player2
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
			],
			vec![
				// diagonal-Coin::Player2
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
			],
			vec![
				// diagonal-Coin::Player2
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
			],
			vec![
				// diagonal-Coin::Player2
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
			],
			vec![
				// diagonal-Coin::Player2
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty],
				vec![Coin::Empty, Coin::Empty, Coin::Empty, Coin::Player1, Coin::Empty, Coin::Empty, Coin::Empty],
			],
		];
		for state in winning_states {
			board.set_state(state);
			assert!(board.has_winner(), "This board should have a winner");
		};
	})
}

