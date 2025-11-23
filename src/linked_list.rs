struct Node<T> {
    data: T,
    next: Option<Box<Node<T>>>,
}

pub struct SinglyLinkedList<T> {
    head: Option<Box<Node<T>>>,
    count: u64,
}

impl<T> SinglyLinkedList<T> {
    /// 创建新的链表
    pub fn new() -> Self {
        Self {
            head: None,
            count: 0,
        }
    }

    /// 在链表尾部插入新元素，时间复杂度 O(1)
    pub fn push(&mut self, data: T) {
        let node = Box::new(Node {
            data,
            next: self.head.take(),
        });
        self.head = Some(node);
        self.count += 1;
    }

    /// 删除链表尾部元素，时间复杂度 O(1)
    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            self.count -= 1;
            node.data
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut list = SinglyLinkedList::new();
        list.push(3);
        list.push(4);
        assert_eq!(list.pop(), Some(4));
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), None);
    }
}
