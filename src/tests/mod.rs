use Maze;

mod data;


#[test]
fn width_correct() {
    let width = 10;
    let height = 5;
    let maze = Maze::<data::TestRoom>::new(width, height);

    assert!(maze.width() == width);
}


#[test]
fn height_correct() {
    let width = 10;
    let height = 5;
    let maze = Maze::<data::TestRoom>::new(width, height);

    assert!(maze.height() == height);
}
