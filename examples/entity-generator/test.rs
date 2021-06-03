impl<Item> IsArray for [Item; 0usize] {
    type Item = Item;
    const LEN: usize = 0usize;
}
impl<Item> IsArray for [Item; 1usize] {
    type Item = Item;
    const LEN: usize = 1usize;
}
impl<Item> IsArray for [Item; 2usize] {
    type Item = Item;
    const LEN: usize = 2usize;
}
impl<Item> IsArray for [Item; 3usize] {
    type Item = Item;
    const LEN: usize = 3usize;
}
impl<Item> IsArray for [Item; 4usize] {
    type Item = Item;
    const LEN: usize = 4usize;
}
