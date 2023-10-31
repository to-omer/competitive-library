(function() {var type_impls = {
"competitive":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-CompressedBinaryIndexedTree%3CM,+A,+Tag%3CM%3E%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_binary_indexed_tree.rs.html#199-209\">source</a><a href=\"#impl-CompressedBinaryIndexedTree%3CM,+A,+Tag%3CM%3E%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;M, A&gt; <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html\" title=\"struct competitive::data_structure::CompressedBinaryIndexedTree\">CompressedBinaryIndexedTree</a>&lt;M, A, Tag&lt;M&gt;&gt;<span class=\"where fmt-newline\">where\n    M: <a class=\"trait\" href=\"competitive/algebra/trait.Monoid.html\" title=\"trait competitive::algebra::Monoid\">Monoid</a>,\n    A: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>,</span></h3></section></summary><div class=\"impl-items\"><section id=\"method.new\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_binary_indexed_tree.rs.html#199-209\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html#tymethod.new\" class=\"fn\">new</a>(points: &amp;[<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(A,)</a>]) -&gt; Self</h4></section><section id=\"method.accumulate\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_binary_indexed_tree.rs.html#199-209\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html#tymethod.accumulate\" class=\"fn\">accumulate</a>&lt;QA&gt;(&amp;self, range: &amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(QA,)</a>) -&gt; M::<a class=\"associatedtype\" href=\"competitive/algebra/trait.Magma.html#associatedtype.T\" title=\"type competitive::algebra::Magma::T\">T</a><span class=\"where fmt-newline\">where\n    QA: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/range/trait.RangeBounds.html\" title=\"trait core::ops::range::RangeBounds\">RangeBounds</a>&lt;A&gt;,</span></h4></section><section id=\"method.update\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_binary_indexed_tree.rs.html#199-209\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html#tymethod.update\" class=\"fn\">update</a>(&amp;mut self, key: &amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(A,)</a>, x: &amp;M::<a class=\"associatedtype\" href=\"competitive/algebra/trait.Magma.html#associatedtype.T\" title=\"type competitive::algebra::Magma::T\">T</a>)</h4></section></div></details>",0,"competitive::data_structure::compressed_binary_indexed_tree::CompressedBinaryIndexedTree1d"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-CompressedBinaryIndexedTree%3CM,+A,+CompressedBinaryIndexedTree%3CM,+B,+Tag%3CM%3E%3E%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_binary_indexed_tree.rs.html#199-209\">source</a><a href=\"#impl-CompressedBinaryIndexedTree%3CM,+A,+CompressedBinaryIndexedTree%3CM,+B,+Tag%3CM%3E%3E%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;M, A, B&gt; <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html\" title=\"struct competitive::data_structure::CompressedBinaryIndexedTree\">CompressedBinaryIndexedTree</a>&lt;M, A, <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html\" title=\"struct competitive::data_structure::CompressedBinaryIndexedTree\">CompressedBinaryIndexedTree</a>&lt;M, B, Tag&lt;M&gt;&gt;&gt;<span class=\"where fmt-newline\">where\n    M: <a class=\"trait\" href=\"competitive/algebra/trait.Monoid.html\" title=\"trait competitive::algebra::Monoid\">Monoid</a>,\n    A: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>,\n    B: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>,</span></h3></section></summary><div class=\"impl-items\"><section id=\"method.new\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_binary_indexed_tree.rs.html#199-209\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html#tymethod.new\" class=\"fn\">new</a>(points: &amp;[(A, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(B,)</a>)]) -&gt; Self</h4></section><section id=\"method.accumulate\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_binary_indexed_tree.rs.html#199-209\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html#tymethod.accumulate\" class=\"fn\">accumulate</a>&lt;QA, QB&gt;(&amp;self, range: &amp;(QA, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(QB,)</a>)) -&gt; M::<a class=\"associatedtype\" href=\"competitive/algebra/trait.Magma.html#associatedtype.T\" title=\"type competitive::algebra::Magma::T\">T</a><span class=\"where fmt-newline\">where\n    QA: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/range/trait.RangeBounds.html\" title=\"trait core::ops::range::RangeBounds\">RangeBounds</a>&lt;A&gt;,\n    QB: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/range/trait.RangeBounds.html\" title=\"trait core::ops::range::RangeBounds\">RangeBounds</a>&lt;B&gt;,</span></h4></section><section id=\"method.update\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_binary_indexed_tree.rs.html#199-209\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html#tymethod.update\" class=\"fn\">update</a>(&amp;mut self, key: &amp;(A, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(B,)</a>), x: &amp;M::<a class=\"associatedtype\" href=\"competitive/algebra/trait.Magma.html#associatedtype.T\" title=\"type competitive::algebra::Magma::T\">T</a>)</h4></section></div></details>",0,"competitive::data_structure::compressed_binary_indexed_tree::CompressedBinaryIndexedTree2d"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-CompressedBinaryIndexedTree%3CM,+A,+CompressedBinaryIndexedTree%3CM,+B,+CompressedBinaryIndexedTree%3CM,+C,+Tag%3CM%3E%3E%3E%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_binary_indexed_tree.rs.html#199-209\">source</a><a href=\"#impl-CompressedBinaryIndexedTree%3CM,+A,+CompressedBinaryIndexedTree%3CM,+B,+CompressedBinaryIndexedTree%3CM,+C,+Tag%3CM%3E%3E%3E%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;M, A, B, C&gt; <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html\" title=\"struct competitive::data_structure::CompressedBinaryIndexedTree\">CompressedBinaryIndexedTree</a>&lt;M, A, <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html\" title=\"struct competitive::data_structure::CompressedBinaryIndexedTree\">CompressedBinaryIndexedTree</a>&lt;M, B, <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html\" title=\"struct competitive::data_structure::CompressedBinaryIndexedTree\">CompressedBinaryIndexedTree</a>&lt;M, C, Tag&lt;M&gt;&gt;&gt;&gt;<span class=\"where fmt-newline\">where\n    M: <a class=\"trait\" href=\"competitive/algebra/trait.Monoid.html\" title=\"trait competitive::algebra::Monoid\">Monoid</a>,\n    A: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>,\n    B: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>,\n    C: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>,</span></h3></section></summary><div class=\"impl-items\"><section id=\"method.new\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_binary_indexed_tree.rs.html#199-209\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html#tymethod.new\" class=\"fn\">new</a>(points: &amp;[(A, (B, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(C,)</a>))]) -&gt; Self</h4></section><section id=\"method.accumulate\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_binary_indexed_tree.rs.html#199-209\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html#tymethod.accumulate\" class=\"fn\">accumulate</a>&lt;QA, QB, QC&gt;(&amp;self, range: &amp;(QA, (QB, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(QC,)</a>))) -&gt; M::<a class=\"associatedtype\" href=\"competitive/algebra/trait.Magma.html#associatedtype.T\" title=\"type competitive::algebra::Magma::T\">T</a><span class=\"where fmt-newline\">where\n    QA: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/range/trait.RangeBounds.html\" title=\"trait core::ops::range::RangeBounds\">RangeBounds</a>&lt;A&gt;,\n    QB: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/range/trait.RangeBounds.html\" title=\"trait core::ops::range::RangeBounds\">RangeBounds</a>&lt;B&gt;,\n    QC: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/range/trait.RangeBounds.html\" title=\"trait core::ops::range::RangeBounds\">RangeBounds</a>&lt;C&gt;,</span></h4></section><section id=\"method.update\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_binary_indexed_tree.rs.html#199-209\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html#tymethod.update\" class=\"fn\">update</a>(&amp;mut self, key: &amp;(A, (B, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(C,)</a>)), x: &amp;M::<a class=\"associatedtype\" href=\"competitive/algebra/trait.Magma.html#associatedtype.T\" title=\"type competitive::algebra::Magma::T\">T</a>)</h4></section></div></details>",0,"competitive::data_structure::compressed_binary_indexed_tree::CompressedBinaryIndexedTree3d"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-CompressedBinaryIndexedTree%3CM,+A,+CompressedBinaryIndexedTree%3CM,+B,+CompressedBinaryIndexedTree%3CM,+C,+CompressedBinaryIndexedTree%3CM,+D,+Tag%3CM%3E%3E%3E%3E%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_binary_indexed_tree.rs.html#199-209\">source</a><a href=\"#impl-CompressedBinaryIndexedTree%3CM,+A,+CompressedBinaryIndexedTree%3CM,+B,+CompressedBinaryIndexedTree%3CM,+C,+CompressedBinaryIndexedTree%3CM,+D,+Tag%3CM%3E%3E%3E%3E%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;M, A, B, C, D&gt; <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html\" title=\"struct competitive::data_structure::CompressedBinaryIndexedTree\">CompressedBinaryIndexedTree</a>&lt;M, A, <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html\" title=\"struct competitive::data_structure::CompressedBinaryIndexedTree\">CompressedBinaryIndexedTree</a>&lt;M, B, <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html\" title=\"struct competitive::data_structure::CompressedBinaryIndexedTree\">CompressedBinaryIndexedTree</a>&lt;M, C, <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html\" title=\"struct competitive::data_structure::CompressedBinaryIndexedTree\">CompressedBinaryIndexedTree</a>&lt;M, D, Tag&lt;M&gt;&gt;&gt;&gt;&gt;<span class=\"where fmt-newline\">where\n    M: <a class=\"trait\" href=\"competitive/algebra/trait.Monoid.html\" title=\"trait competitive::algebra::Monoid\">Monoid</a>,\n    A: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>,\n    B: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>,\n    C: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>,\n    D: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>,</span></h3></section></summary><div class=\"impl-items\"><section id=\"method.new\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_binary_indexed_tree.rs.html#199-209\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html#tymethod.new\" class=\"fn\">new</a>(points: &amp;[(A, (B, (C, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(D,)</a>)))]) -&gt; Self</h4></section><section id=\"method.accumulate\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_binary_indexed_tree.rs.html#199-209\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html#tymethod.accumulate\" class=\"fn\">accumulate</a>&lt;QA, QB, QC, QD&gt;(\n    &amp;self,\n    range: &amp;(QA, (QB, (QC, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(QD,)</a>)))\n) -&gt; M::<a class=\"associatedtype\" href=\"competitive/algebra/trait.Magma.html#associatedtype.T\" title=\"type competitive::algebra::Magma::T\">T</a><span class=\"where fmt-newline\">where\n    QA: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/range/trait.RangeBounds.html\" title=\"trait core::ops::range::RangeBounds\">RangeBounds</a>&lt;A&gt;,\n    QB: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/range/trait.RangeBounds.html\" title=\"trait core::ops::range::RangeBounds\">RangeBounds</a>&lt;B&gt;,\n    QC: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/range/trait.RangeBounds.html\" title=\"trait core::ops::range::RangeBounds\">RangeBounds</a>&lt;C&gt;,\n    QD: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/range/trait.RangeBounds.html\" title=\"trait core::ops::range::RangeBounds\">RangeBounds</a>&lt;D&gt;,</span></h4></section><section id=\"method.update\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_binary_indexed_tree.rs.html#199-209\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html#tymethod.update\" class=\"fn\">update</a>(&amp;mut self, key: &amp;(A, (B, (C, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(D,)</a>))), x: &amp;M::<a class=\"associatedtype\" href=\"competitive/algebra/trait.Magma.html#associatedtype.T\" title=\"type competitive::algebra::Magma::T\">T</a>)</h4></section></div></details>",0,"competitive::data_structure::compressed_binary_indexed_tree::CompressedBinaryIndexedTree4d"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Clone-for-CompressedBinaryIndexedTree%3CM,+X,+Inner%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_binary_indexed_tree.rs.html#29-42\">source</a><a href=\"#impl-Clone-for-CompressedBinaryIndexedTree%3CM,+X,+Inner%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;M, X, Inner&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html\" title=\"struct competitive::data_structure::CompressedBinaryIndexedTree\">CompressedBinaryIndexedTree</a>&lt;M, X, Inner&gt;<span class=\"where fmt-newline\">where\n    M: <a class=\"trait\" href=\"competitive/algebra/trait.Monoid.html\" title=\"trait competitive::algebra::Monoid\">Monoid</a>,\n    X: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,\n    Inner: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,</span></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_binary_indexed_tree.rs.html#35-41\">source</a><a href=\"#method.clone\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone\" class=\"fn\">clone</a>(&amp;self) -&gt; Self</h4></section></summary><div class='docblock'>Returns a copy of the value. <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone_from\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/clone.rs.html#169\">source</a></span><a href=\"#method.clone_from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from\" class=\"fn\">clone_from</a>(&amp;mut self, source: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;Self</a>)</h4></section></summary><div class='docblock'>Performs copy-assignment from <code>source</code>. <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from\">Read more</a></div></details></div></details>","Clone","competitive::data_structure::compressed_binary_indexed_tree::CompressedBinaryIndexedTree1d","competitive::data_structure::compressed_binary_indexed_tree::CompressedBinaryIndexedTree2d","competitive::data_structure::compressed_binary_indexed_tree::CompressedBinaryIndexedTree3d","competitive::data_structure::compressed_binary_indexed_tree::CompressedBinaryIndexedTree4d"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Default-for-CompressedBinaryIndexedTree%3CM,+X,+Inner%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_binary_indexed_tree.rs.html#43-54\">source</a><a href=\"#impl-Default-for-CompressedBinaryIndexedTree%3CM,+X,+Inner%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;M, X, Inner&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a> for <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html\" title=\"struct competitive::data_structure::CompressedBinaryIndexedTree\">CompressedBinaryIndexedTree</a>&lt;M, X, Inner&gt;<span class=\"where fmt-newline\">where\n    M: <a class=\"trait\" href=\"competitive/algebra/trait.Monoid.html\" title=\"trait competitive::algebra::Monoid\">Monoid</a>,</span></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.default\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_binary_indexed_tree.rs.html#47-53\">source</a><a href=\"#method.default\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/default/trait.Default.html#tymethod.default\" class=\"fn\">default</a>() -&gt; Self</h4></section></summary><div class='docblock'>Returns the “default value” for a type. <a href=\"https://doc.rust-lang.org/nightly/core/default/trait.Default.html#tymethod.default\">Read more</a></div></details></div></details>","Default","competitive::data_structure::compressed_binary_indexed_tree::CompressedBinaryIndexedTree1d","competitive::data_structure::compressed_binary_indexed_tree::CompressedBinaryIndexedTree2d","competitive::data_structure::compressed_binary_indexed_tree::CompressedBinaryIndexedTree3d","competitive::data_structure::compressed_binary_indexed_tree::CompressedBinaryIndexedTree4d"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-CompressedBinaryIndexedTree%3CM,+X,+Inner%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_binary_indexed_tree.rs.html#16-28\">source</a><a href=\"#impl-Debug-for-CompressedBinaryIndexedTree%3CM,+X,+Inner%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;M, X, Inner&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedBinaryIndexedTree.html\" title=\"struct competitive::data_structure::CompressedBinaryIndexedTree\">CompressedBinaryIndexedTree</a>&lt;M, X, Inner&gt;<span class=\"where fmt-newline\">where\n    M: <a class=\"trait\" href=\"competitive/algebra/trait.Monoid.html\" title=\"trait competitive::algebra::Monoid\">Monoid</a>,\n    X: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>,\n    Inner: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>,</span></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_binary_indexed_tree.rs.html#22-27\">source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/nightly/core/fmt/type.Result.html\" title=\"type core::fmt::Result\">Result</a></h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","competitive::data_structure::compressed_binary_indexed_tree::CompressedBinaryIndexedTree1d","competitive::data_structure::compressed_binary_indexed_tree::CompressedBinaryIndexedTree2d","competitive::data_structure::compressed_binary_indexed_tree::CompressedBinaryIndexedTree3d","competitive::data_structure::compressed_binary_indexed_tree::CompressedBinaryIndexedTree4d"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()