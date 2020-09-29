use crate::ffi::c_str;
use avahi_sys::{
    avahi_free, avahi_string_list_add_pair, avahi_string_list_copy, avahi_string_list_equal,
    avahi_string_list_find, avahi_string_list_free, avahi_string_list_get_next,
    avahi_string_list_get_pair, avahi_string_list_length, avahi_string_list_new,
    avahi_string_list_to_string, AvahiStringList,
};
use libc::{c_char, c_void};
use std::marker::PhantomData;
use std::ptr;

#[derive(Debug)]
pub struct ManagedAvahiStringList(*mut AvahiStringList);

impl ManagedAvahiStringList {
    pub fn new() -> Self {
        Self(unsafe { avahi_string_list_new(ptr::null()) })
    }

    pub unsafe fn add_pair(&mut self, key: *const c_char, value: *const c_char) {
        self.0 = avahi_string_list_add_pair(self.0, key, value);
    }

    pub unsafe fn find(&mut self, key: *const c_char) -> Option<AvahiStringListNode> {
        let node = avahi_string_list_find(self.0, key);
        if !node.is_null() {
            Some(AvahiStringListNode::new(node))
        } else {
            None
        }
    }

    pub fn head(&mut self) -> AvahiStringListNode {
        AvahiStringListNode::new(self.0)
    }

    pub fn length(&self) -> u32 {
        unsafe { avahi_string_list_length(self.0) }
    }

    pub fn to_string(&self) -> AvahiString {
        unsafe { avahi_string_list_to_string(self.0).into() }
    }
}

impl Clone for ManagedAvahiStringList {
    fn clone(&self) -> Self {
        Self(unsafe { avahi_string_list_copy(self.0) })
    }
}

impl PartialEq for ManagedAvahiStringList {
    fn eq(&self, other: &Self) -> bool {
        unsafe { avahi_string_list_equal(self.0, other.0) == 1 }
    }
}

impl Eq for ManagedAvahiStringList {}

impl Default for ManagedAvahiStringList {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ManagedAvahiStringList {
    fn drop(&mut self) {
        unsafe { avahi_string_list_free(self.0) };
    }
}

#[derive(new)]
pub struct AvahiStringListNode<'a> {
    list: *mut AvahiStringList,
    phantom: PhantomData<&'a AvahiStringList>,
}

impl<'a> AvahiStringListNode<'a> {
    pub fn next(self) -> Option<AvahiStringListNode<'a>> {
        let next = unsafe { avahi_string_list_get_next(self.list) };
        if next.is_null() {
            None
        } else {
            Some(AvahiStringListNode::new(next))
        }
    }

    pub fn get_pair(&mut self) -> AvahiPair {
        let mut key: *mut c_char = ptr::null_mut();
        let mut value: *mut c_char = ptr::null_mut();
        let mut value_size: usize = 0;

        unsafe {
            avahi_string_list_get_pair(self.list, &mut key, &mut value, &mut value_size);
        }

        AvahiPair::new(key.into(), value.into(), value_size)
    }
}

#[derive(new, Getters)]
pub struct AvahiPair {
    key: AvahiString,
    value: AvahiString,
    value_size: usize,
}

#[derive(new)]
pub struct AvahiString(*mut c_char);

impl AvahiString {
    pub fn as_str(&self) -> Option<&str> {
        if self.0.is_null() {
            None
        } else {
            Some(unsafe { c_str::raw_to_str(self.0) })
        }
    }
}

impl From<*mut c_char> for AvahiString {
    fn from(s: *mut c_char) -> Self {
        Self::new(s)
    }
}

impl Drop for AvahiString {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { avahi_free(self.0 as *mut c_void) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn add_get_pair_success() {
        crate::tests::setup();

        let mut list = ManagedAvahiStringList::new();
        let key1 = c_string!("foo");
        let value1 = c_string!("bar");

        unsafe {
            list.add_pair(
                key1.as_ptr() as *const c_char,
                value1.as_ptr() as *const c_char,
            );

            let key2 = c_string!("hello");
            let value2 = c_string!("world");

            list.add_pair(
                key2.as_ptr() as *const c_char,
                value2.as_ptr() as *const c_char,
            );

            let pair1 = list
                .find(key1.as_ptr() as *const c_char)
                .unwrap()
                .get_pair();

            let pair2 = list
                .find(key2.as_ptr() as *const c_char)
                .unwrap()
                .get_pair();

            assert_eq!(pair1.key().as_str().unwrap(), "foo");
            assert_eq!(pair1.value().as_str().unwrap(), "bar");
            assert_eq!(pair2.key().as_str().unwrap(), "hello");
            assert_eq!(pair2.value().as_str().unwrap(), "world");
        }
    }

    #[test]
    fn add_pair_replaces_success() {
        crate::tests::setup();

        let mut list = ManagedAvahiStringList::new();
        let key = c_string!("foo");
        let value = c_string!("bar");

        unsafe {
            list.add_pair(
                key.as_ptr() as *const c_char,
                value.as_ptr() as *const c_char,
            );

            let pair = list.find(key.as_ptr() as *const c_char).unwrap().get_pair();

            assert_eq!(pair.value().as_str().unwrap(), "bar");

            let value = c_string!("baz");

            list.add_pair(
                key.as_ptr() as *const c_char,
                value.as_ptr() as *const c_char,
            );

            let pair = list.find(key.as_ptr() as *const c_char).unwrap().get_pair();

            assert_eq!(pair.value().as_str().unwrap(), "baz");
        }
    }

    #[test]
    fn length_success() {
        crate::tests::setup();

        let mut list = ManagedAvahiStringList::new();
        let key = c_string!("foo");
        let value = c_string!("bar");

        unsafe {
            list.add_pair(
                key.as_ptr() as *const c_char,
                value.as_ptr() as *const c_char,
            );

            assert_eq!(list.length(), 1);
        }
    }

    #[test]
    fn to_string_success() {
        crate::tests::setup();

        let mut list = ManagedAvahiStringList::new();
        let key = c_string!("foo");
        let value = c_string!("bar");

        unsafe {
            list.add_pair(
                key.as_ptr() as *const c_char,
                value.as_ptr() as *const c_char,
            );

            assert_eq!(list.to_string().as_str().unwrap(), "\"foo=bar\"");
        }
    }

    #[test]
    fn equals_success() {
        crate::tests::setup();

        let mut list = ManagedAvahiStringList::new();
        let key = c_string!("foo");
        let value = c_string!("bar");

        unsafe {
            list.add_pair(
                key.as_ptr() as *const c_char,
                value.as_ptr() as *const c_char,
            );

            assert_eq!(list.clone(), list);
        }
    }

    #[test]
    fn iterate_success() {
        crate::tests::setup();

        let mut list = ManagedAvahiStringList::new();
        let key1 = c_string!("foo");
        let value1 = c_string!("bar");
        let key2 = c_string!("hello");
        let value2 = c_string!("world");

        unsafe {
            list.add_pair(
                key1.as_ptr() as *const c_char,
                value1.as_ptr() as *const c_char,
            );

            list.add_pair(
                key2.as_ptr() as *const c_char,
                value2.as_ptr() as *const c_char,
            );
        }

        let mut node = Some(list.head());
        let mut map = HashMap::new();

        while node.is_some() {
            let mut n = node.unwrap();
            let pair = n.get_pair();

            map.insert(
                pair.key().as_str().unwrap().to_string(),
                pair.value().as_str().unwrap().to_string(),
            );

            node = n.next();
        }

        let expected: HashMap<String, String> = hashmap! {
            "foo".to_string() => "bar".to_string(),
            "hello".to_string() => "world".to_string()
        };

        assert_eq!(map, expected);
    }
}
