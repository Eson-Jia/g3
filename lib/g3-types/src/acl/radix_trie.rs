/*
 * Copyright 2023 ByteDance and/or its affiliates.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::borrow::Borrow;
use std::hash::Hash;

use ahash::AHashMap;
use radix_trie::{Trie, TrieKey};

use super::{AclAction, ActionContract};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AclRadixTrieRuleBuilder<K, Action = AclAction>
where
    K: TrieKey + Hash,
{
    inner: AHashMap<K, Action>,
    missed_action: Action,
}

impl<K, Action> AclRadixTrieRuleBuilder<K, Action>
where
    K: TrieKey + Hash + Clone,
    Action: ActionContract,
{
    pub fn new(missed_action: Action) -> Self {
        AclRadixTrieRuleBuilder {
            inner: AHashMap::new(),
            missed_action,
        }
    }

    #[inline]
    pub fn add_node(&mut self, node: K, action: Action) {
        self.inner.insert(node, action);
    }

    #[inline]
    pub fn set_missed_action(&mut self, action: Action) {
        self.missed_action = action;
    }

    #[inline]
    pub fn missed_action(&self) -> Action {
        self.missed_action
    }

    pub fn build(&self) -> AclRadixTrieRule<K, Action> {
        let mut trie = Trie::new();

        for (k, v) in &self.inner {
            trie.insert(k.clone(), *v);
        }

        AclRadixTrieRule {
            inner: trie,
            missed_action: self.missed_action,
        }
    }
}

pub struct AclRadixTrieRule<K: TrieKey, Action = AclAction> {
    inner: Trie<K, Action>,
    missed_action: Action,
}

impl<K: TrieKey, Action: ActionContract> AclRadixTrieRule<K, Action> {
    pub fn check<Q>(&self, key: &Q) -> (bool, Action)
    where
        K: Borrow<Q>,
        Q: TrieKey,
    {
        if let Some(action) = self.inner.get_ancestor_value(key) {
            (true, *action)
        } else {
            (false, self.missed_action)
        }
    }
}
