use std::{collections::HashMap, rc::Rc};

type Link<T> = Option<Rc<Node<T>>>;

#[derive(Debug)]
pub struct Node<T> {
    val: T,
    next: Link<T>,
}

#[derive(Debug)]
struct LinkedList<T> {
    head: Link<T>,
    tail: Link<T>,
}

impl<T> LinkedList<T> {
    fn new() -> Self {
        LinkedList {
            head: None,
            tail: None,
        }
    }
}

#[derive(Debug)]
pub struct HashWheel<T> {
    table: HashMap<T, Node<T>>,
    list: LinkedList<T>,
}

impl<T> HashWheel<T> {
    pub fn tick(&mut self) {}
}
