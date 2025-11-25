use std::cell::{Ref, RefCell};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;

struct Node<K, V> {
    key: K,
    val: V,
    list_prev: Option<Rc<RefCell<Node<K, V>>>>,
    list_next: Option<Rc<RefCell<Node<K, V>>>>,
    hash_prev: Option<Rc<RefCell<Node<K, V>>>>,
    hash_next: Option<Rc<RefCell<Node<K, V>>>>,
}

impl<K, V> Node<K, V> {
    fn new(key: K, val: V) -> Self {
        Self {
            key,
            val,
            list_prev: None,
            list_next: None,
            hash_prev: None,
            hash_next: None,
        }
    }
}

pub struct LRUCache<K, V, const H: usize = 100> {
    max_size: usize,
    total_size: usize,
    list_header: Option<Rc<RefCell<Node<K, V>>>>,
    list_tail: Option<Rc<RefCell<Node<K, V>>>>,
    hash_indices: Box<[Option<Rc<RefCell<Node<K, V>>>>; H]>,
}

impl<K: Hash + PartialEq<K>, V, const H: usize> LRUCache<K, V, H> {
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            total_size: 0,
            list_header: None,
            list_tail: None,
            hash_indices: Box::new([const { None }; H]),
        }
    }

    pub fn get<T, F>(&mut self, key: T, then: F)
    where
        T: std::borrow::Borrow<K>,
        F: Fn(&V),
    {
        if let Some(node) = self.find_node_by_key(key.borrow()) {
            self.move_node_to_list_head(node.clone());
            then(&node.borrow().val)
        }
    }

    pub fn try_get<T: std::borrow::Borrow<K>>(&mut self, key: T) -> Option<V>
    where
        V: Clone,
    {
        if let Some(node) = self.find_node_by_key(key.borrow()) {
            self.move_node_to_list_head(node.clone());
            Some(node.borrow().val.clone())
        } else {
            None
        }
    }

    pub fn set(&mut self, key: K, val: V) {
        if let Some(node) = self.find_node_by_key(&key) {
            self.move_node_to_list_head(node.clone());
            node.borrow_mut().val = val;
        } else {
            let node = Rc::new(RefCell::new(Node::new(key, val)));

            if let Some(list_header) = self.list_header.clone() {
                list_header.borrow_mut().list_prev = Some(node.clone());
            }
            node.borrow_mut().list_next = self.list_header.clone();
            self.list_header = Some(node.clone());

            let key_hash = Self::get_key_hash(&node.borrow().key);
            if let Some(Some(index)) = self.hash_indices.get(key_hash) {
                index.borrow_mut().hash_prev = Some(node);
                self.hash_indices[key_hash] = Some(index.clone());
            } else {
                self.hash_indices[key_hash] = Some(node.clone());
            }

            if self.total_size >= self.max_size {
                self.remove_list_tail();
            } else {
                self.total_size += 1;
            }
        }
    }

    fn move_node_to_list_head(&mut self, node: Rc<RefCell<Node<K, V>>>) {
        let prev_node = node.borrow().list_prev.clone();
        let next_node = node.borrow().list_next.clone();

        if let Some(prev_node) = prev_node.as_ref() {
            prev_node.borrow_mut().list_next = next_node.clone();
        }

        if let Some(next_node) = next_node.as_ref() {
            next_node.borrow_mut().list_prev = prev_node.clone();
        }

        node.borrow_mut().list_prev = None;
        node.borrow_mut().list_next = self.list_header.clone();
        self.list_header = Some(node.clone());
    }

    fn remove_list_tail(&mut self) {
        if let Some(list_tail) = self.list_tail.clone() {
            if let Some(prev_node) = list_tail.borrow().list_prev.clone() {
                prev_node.borrow_mut().list_next = None;
                self.list_tail = Some(prev_node);
            } else {
                self.list_tail = None;
            }

            let prev_node = list_tail.borrow().hash_prev.clone();
            let next_node = list_tail.borrow().hash_next.clone();
            if let Some(prev_node) = prev_node.clone() {
                prev_node.borrow_mut().hash_next = next_node.clone();
            } else {
                let key_hash = Self::get_key_hash(&list_tail.borrow().key);
                self.hash_indices[key_hash] = next_node.clone();
            }

            if let Some(next_node) = next_node.clone() {
                next_node.borrow_mut().hash_prev = prev_node.clone();
            }
        }
    }

    fn find_node_by_key(&self, key: &K) -> Option<Rc<RefCell<Node<K, V>>>> {
        let key_hash = Self::get_key_hash(key);

        if let Some(index) = self.hash_indices.get(key_hash) {
            let mut p = index.clone();
            while let Some(ptr) = p {
                if ptr.borrow().key == *key {
                    return Some(ptr);
                }
                p = ptr.borrow().hash_next.clone();
            }
        }

        None
    }

    fn get_key_hash(key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize % H
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[test]
    fn test_01() {
        let mut cache: LRUCache<i32, i32> = LRUCache::new(10);

        cache.set(1, 1);
        cache.set(2, 2);
        cache.set(2, 3);

        cache.get(1, |v| println!("get: {:?}", v));
        cache.get(2, |v| println!("get: {:?}", v));

        println!("total_size: {:?}", cache.total_size);
    }

    #[test]
    fn test_02() {
        let mut cache: LRUCache<&str, Point> = LRUCache::new(10);

        for i in 0..10 {
            cache.set("1", Point { x: 1, y: 1 });
            cache.set("2", Point { x: 2, y: 2 });
        }

        cache.get("1", |v| println!("get: {:?}", v));
        cache.get("2", |v| println!("get: {:?}", v));

        println!("total_size: {:?}", cache.total_size);
    }
}
