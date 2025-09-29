#[derive(Copy, Clone, Debug)]
pub enum MenuTabs {
    Queue,
    Directories,
    Artists,
    Albums,
    Playlists,
    Search,
}

impl From<MenuTabs> for usize {
    fn from(input: MenuTabs) -> usize {
        match input {
            MenuTabs::Queue => 0,
            MenuTabs::Directories => 1,
            MenuTabs::Artists => 2,
            MenuTabs::Albums => 3,
            MenuTabs::Playlists => 4,
            MenuTabs::Search => 5,
        }
    }
}
