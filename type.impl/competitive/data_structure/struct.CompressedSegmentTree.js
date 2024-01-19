(function() {var type_impls = {
"competitive":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-CompressedSegmentTree%3CM,+T1,+Tag%3CM%3E%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_segment_tree.rs.html#214-224\">source</a><a href=\"#impl-CompressedSegmentTree%3CM,+T1,+Tag%3CM%3E%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;M, T1&gt; <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedSegmentTree.html\" title=\"struct competitive::data_structure::CompressedSegmentTree\">CompressedSegmentTree</a>&lt;M, T1, Tag&lt;M&gt;&gt;<div class=\"where\">where\n    M: <a class=\"trait\" href=\"competitive/algebra/trait.Monoid.html\" title=\"trait competitive::algebra::Monoid\">Monoid</a>,\n    T1: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>,</div></h3></section></summary><div class=\"impl-items\"><section id=\"method.new\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_segment_tree.rs.html#214-224\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedSegmentTree.html#tymethod.new\" class=\"fn\">new</a>(points: &amp;[<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(T1,)</a>]) -&gt; Self</h4></section><section id=\"method.fold\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_segment_tree.rs.html#214-224\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedSegmentTree.html#tymethod.fold\" class=\"fn\">fold</a>&lt;Q1&gt;(&amp;self, range: &amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(Q1,)</a>) -&gt; M::<a class=\"associatedtype\" href=\"competitive/algebra/trait.Magma.html#associatedtype.T\" title=\"type competitive::algebra::Magma::T\">T</a><div class=\"where\">where\n    Q1: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/range/trait.RangeBounds.html\" title=\"trait core::ops::range::RangeBounds\">RangeBounds</a>&lt;T1&gt;,</div></h4></section><section id=\"method.update\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_segment_tree.rs.html#214-224\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedSegmentTree.html#tymethod.update\" class=\"fn\">update</a>(&amp;mut self, key: &amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(T1,)</a>, x: &amp;M::<a class=\"associatedtype\" href=\"competitive/algebra/trait.Magma.html#associatedtype.T\" title=\"type competitive::algebra::Magma::T\">T</a>)</h4></section></div></details>",0,"competitive::data_structure::compressed_segment_tree::CompressedSegmentTree1d"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-CompressedSegmentTree%3CM,+T1,+CompressedSegmentTree%3CM,+T2,+Tag%3CM%3E%3E%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_segment_tree.rs.html#214-224\">source</a><a href=\"#impl-CompressedSegmentTree%3CM,+T1,+CompressedSegmentTree%3CM,+T2,+Tag%3CM%3E%3E%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;M, T1, T2&gt; <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedSegmentTree.html\" title=\"struct competitive::data_structure::CompressedSegmentTree\">CompressedSegmentTree</a>&lt;M, T1, <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedSegmentTree.html\" title=\"struct competitive::data_structure::CompressedSegmentTree\">CompressedSegmentTree</a>&lt;M, T2, Tag&lt;M&gt;&gt;&gt;<div class=\"where\">where\n    M: <a class=\"trait\" href=\"competitive/algebra/trait.Monoid.html\" title=\"trait competitive::algebra::Monoid\">Monoid</a>,\n    T1: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>,\n    T2: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>,</div></h3></section></summary><div class=\"impl-items\"><section id=\"method.new\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_segment_tree.rs.html#214-224\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedSegmentTree.html#tymethod.new\" class=\"fn\">new</a>(points: &amp;[(T1, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(T2,)</a>)]) -&gt; Self</h4></section><section id=\"method.fold\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_segment_tree.rs.html#214-224\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedSegmentTree.html#tymethod.fold\" class=\"fn\">fold</a>&lt;Q1, Q2&gt;(&amp;self, range: &amp;(Q1, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(Q2,)</a>)) -&gt; M::<a class=\"associatedtype\" href=\"competitive/algebra/trait.Magma.html#associatedtype.T\" title=\"type competitive::algebra::Magma::T\">T</a><div class=\"where\">where\n    Q1: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/range/trait.RangeBounds.html\" title=\"trait core::ops::range::RangeBounds\">RangeBounds</a>&lt;T1&gt;,\n    Q2: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/range/trait.RangeBounds.html\" title=\"trait core::ops::range::RangeBounds\">RangeBounds</a>&lt;T2&gt;,</div></h4></section><section id=\"method.update\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_segment_tree.rs.html#214-224\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedSegmentTree.html#tymethod.update\" class=\"fn\">update</a>(&amp;mut self, key: &amp;(T1, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(T2,)</a>), x: &amp;M::<a class=\"associatedtype\" href=\"competitive/algebra/trait.Magma.html#associatedtype.T\" title=\"type competitive::algebra::Magma::T\">T</a>)</h4></section></div></details>",0,"competitive::data_structure::compressed_segment_tree::CompressedSegmentTree2d"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-CompressedSegmentTree%3CM,+T1,+CompressedSegmentTree%3CM,+T2,+CompressedSegmentTree%3CM,+T3,+Tag%3CM%3E%3E%3E%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_segment_tree.rs.html#214-224\">source</a><a href=\"#impl-CompressedSegmentTree%3CM,+T1,+CompressedSegmentTree%3CM,+T2,+CompressedSegmentTree%3CM,+T3,+Tag%3CM%3E%3E%3E%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;M, T1, T2, T3&gt; <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedSegmentTree.html\" title=\"struct competitive::data_structure::CompressedSegmentTree\">CompressedSegmentTree</a>&lt;M, T1, <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedSegmentTree.html\" title=\"struct competitive::data_structure::CompressedSegmentTree\">CompressedSegmentTree</a>&lt;M, T2, <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedSegmentTree.html\" title=\"struct competitive::data_structure::CompressedSegmentTree\">CompressedSegmentTree</a>&lt;M, T3, Tag&lt;M&gt;&gt;&gt;&gt;<div class=\"where\">where\n    M: <a class=\"trait\" href=\"competitive/algebra/trait.Monoid.html\" title=\"trait competitive::algebra::Monoid\">Monoid</a>,\n    T1: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>,\n    T2: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>,\n    T3: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>,</div></h3></section></summary><div class=\"impl-items\"><section id=\"method.new\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_segment_tree.rs.html#214-224\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedSegmentTree.html#tymethod.new\" class=\"fn\">new</a>(points: &amp;[(T1, (T2, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(T3,)</a>))]) -&gt; Self</h4></section><section id=\"method.fold\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_segment_tree.rs.html#214-224\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedSegmentTree.html#tymethod.fold\" class=\"fn\">fold</a>&lt;Q1, Q2, Q3&gt;(&amp;self, range: &amp;(Q1, (Q2, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(Q3,)</a>))) -&gt; M::<a class=\"associatedtype\" href=\"competitive/algebra/trait.Magma.html#associatedtype.T\" title=\"type competitive::algebra::Magma::T\">T</a><div class=\"where\">where\n    Q1: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/range/trait.RangeBounds.html\" title=\"trait core::ops::range::RangeBounds\">RangeBounds</a>&lt;T1&gt;,\n    Q2: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/range/trait.RangeBounds.html\" title=\"trait core::ops::range::RangeBounds\">RangeBounds</a>&lt;T2&gt;,\n    Q3: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/range/trait.RangeBounds.html\" title=\"trait core::ops::range::RangeBounds\">RangeBounds</a>&lt;T3&gt;,</div></h4></section><section id=\"method.update\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_segment_tree.rs.html#214-224\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedSegmentTree.html#tymethod.update\" class=\"fn\">update</a>(&amp;mut self, key: &amp;(T1, (T2, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(T3,)</a>)), x: &amp;M::<a class=\"associatedtype\" href=\"competitive/algebra/trait.Magma.html#associatedtype.T\" title=\"type competitive::algebra::Magma::T\">T</a>)</h4></section></div></details>",0,"competitive::data_structure::compressed_segment_tree::CompressedSegmentTree3d"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-CompressedSegmentTree%3CM,+T1,+CompressedSegmentTree%3CM,+T2,+CompressedSegmentTree%3CM,+T3,+CompressedSegmentTree%3CM,+T4,+Tag%3CM%3E%3E%3E%3E%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_segment_tree.rs.html#214-224\">source</a><a href=\"#impl-CompressedSegmentTree%3CM,+T1,+CompressedSegmentTree%3CM,+T2,+CompressedSegmentTree%3CM,+T3,+CompressedSegmentTree%3CM,+T4,+Tag%3CM%3E%3E%3E%3E%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;M, T1, T2, T3, T4&gt; <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedSegmentTree.html\" title=\"struct competitive::data_structure::CompressedSegmentTree\">CompressedSegmentTree</a>&lt;M, T1, <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedSegmentTree.html\" title=\"struct competitive::data_structure::CompressedSegmentTree\">CompressedSegmentTree</a>&lt;M, T2, <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedSegmentTree.html\" title=\"struct competitive::data_structure::CompressedSegmentTree\">CompressedSegmentTree</a>&lt;M, T3, <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedSegmentTree.html\" title=\"struct competitive::data_structure::CompressedSegmentTree\">CompressedSegmentTree</a>&lt;M, T4, Tag&lt;M&gt;&gt;&gt;&gt;&gt;<div class=\"where\">where\n    M: <a class=\"trait\" href=\"competitive/algebra/trait.Monoid.html\" title=\"trait competitive::algebra::Monoid\">Monoid</a>,\n    T1: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>,\n    T2: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>,\n    T3: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>,\n    T4: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>,</div></h3></section></summary><div class=\"impl-items\"><section id=\"method.new\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_segment_tree.rs.html#214-224\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedSegmentTree.html#tymethod.new\" class=\"fn\">new</a>(points: &amp;[(T1, (T2, (T3, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(T4,)</a>)))]) -&gt; Self</h4></section><section id=\"method.fold\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_segment_tree.rs.html#214-224\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedSegmentTree.html#tymethod.fold\" class=\"fn\">fold</a>&lt;Q1, Q2, Q3, Q4&gt;(&amp;self, range: &amp;(Q1, (Q2, (Q3, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(Q4,)</a>)))) -&gt; M::<a class=\"associatedtype\" href=\"competitive/algebra/trait.Magma.html#associatedtype.T\" title=\"type competitive::algebra::Magma::T\">T</a><div class=\"where\">where\n    Q1: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/range/trait.RangeBounds.html\" title=\"trait core::ops::range::RangeBounds\">RangeBounds</a>&lt;T1&gt;,\n    Q2: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/range/trait.RangeBounds.html\" title=\"trait core::ops::range::RangeBounds\">RangeBounds</a>&lt;T2&gt;,\n    Q3: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/range/trait.RangeBounds.html\" title=\"trait core::ops::range::RangeBounds\">RangeBounds</a>&lt;T3&gt;,\n    Q4: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/range/trait.RangeBounds.html\" title=\"trait core::ops::range::RangeBounds\">RangeBounds</a>&lt;T4&gt;,</div></h4></section><section id=\"method.update\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_segment_tree.rs.html#214-224\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/data_structure/struct.CompressedSegmentTree.html#tymethod.update\" class=\"fn\">update</a>(&amp;mut self, key: &amp;(T1, (T2, (T3, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.tuple.html\">(T4,)</a>))), x: &amp;M::<a class=\"associatedtype\" href=\"competitive/algebra/trait.Magma.html#associatedtype.T\" title=\"type competitive::algebra::Magma::T\">T</a>)</h4></section></div></details>",0,"competitive::data_structure::compressed_segment_tree::CompressedSegmentTree4d"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-CompressedSegmentTree%3CM,+X,+Inner%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_segment_tree.rs.html#18-30\">source</a><a href=\"#impl-Debug-for-CompressedSegmentTree%3CM,+X,+Inner%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;M, X, Inner&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedSegmentTree.html\" title=\"struct competitive::data_structure::CompressedSegmentTree\">CompressedSegmentTree</a>&lt;M, X, Inner&gt;<div class=\"where\">where\n    M: <a class=\"trait\" href=\"competitive/algebra/trait.Monoid.html\" title=\"trait competitive::algebra::Monoid\">Monoid</a>,\n    X: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>,\n    Inner: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_segment_tree.rs.html#24-29\">source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/nightly/core/fmt/type.Result.html\" title=\"type core::fmt::Result\">Result</a></h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","competitive::data_structure::compressed_segment_tree::CompressedSegmentTree1d","competitive::data_structure::compressed_segment_tree::CompressedSegmentTree2d","competitive::data_structure::compressed_segment_tree::CompressedSegmentTree3d","competitive::data_structure::compressed_segment_tree::CompressedSegmentTree4d"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Default-for-CompressedSegmentTree%3CM,+X,+Inner%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_segment_tree.rs.html#47-58\">source</a><a href=\"#impl-Default-for-CompressedSegmentTree%3CM,+X,+Inner%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;M, X, Inner&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a> for <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedSegmentTree.html\" title=\"struct competitive::data_structure::CompressedSegmentTree\">CompressedSegmentTree</a>&lt;M, X, Inner&gt;<div class=\"where\">where\n    M: <a class=\"trait\" href=\"competitive/algebra/trait.Monoid.html\" title=\"trait competitive::algebra::Monoid\">Monoid</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.default\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_segment_tree.rs.html#51-57\">source</a><a href=\"#method.default\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/default/trait.Default.html#tymethod.default\" class=\"fn\">default</a>() -&gt; Self</h4></section></summary><div class='docblock'>Returns the “default value” for a type. <a href=\"https://doc.rust-lang.org/nightly/core/default/trait.Default.html#tymethod.default\">Read more</a></div></details></div></details>","Default","competitive::data_structure::compressed_segment_tree::CompressedSegmentTree1d","competitive::data_structure::compressed_segment_tree::CompressedSegmentTree2d","competitive::data_structure::compressed_segment_tree::CompressedSegmentTree3d","competitive::data_structure::compressed_segment_tree::CompressedSegmentTree4d"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Clone-for-CompressedSegmentTree%3CM,+X,+Inner%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_segment_tree.rs.html#32-45\">source</a><a href=\"#impl-Clone-for-CompressedSegmentTree%3CM,+X,+Inner%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;M, X, Inner&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"competitive/data_structure/struct.CompressedSegmentTree.html\" title=\"struct competitive::data_structure::CompressedSegmentTree\">CompressedSegmentTree</a>&lt;M, X, Inner&gt;<div class=\"where\">where\n    M: <a class=\"trait\" href=\"competitive/algebra/trait.Monoid.html\" title=\"trait competitive::algebra::Monoid\">Monoid</a>,\n    X: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,\n    Inner: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/competitive/data_structure/compressed_segment_tree.rs.html#38-44\">source</a><a href=\"#method.clone\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone\" class=\"fn\">clone</a>(&amp;self) -&gt; Self</h4></section></summary><div class='docblock'>Returns a copy of the value. <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone_from\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/clone.rs.html#169\">source</a></span><a href=\"#method.clone_from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from\" class=\"fn\">clone_from</a>(&amp;mut self, source: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;Self</a>)</h4></section></summary><div class='docblock'>Performs copy-assignment from <code>source</code>. <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from\">Read more</a></div></details></div></details>","Clone","competitive::data_structure::compressed_segment_tree::CompressedSegmentTree1d","competitive::data_structure::compressed_segment_tree::CompressedSegmentTree2d","competitive::data_structure::compressed_segment_tree::CompressedSegmentTree3d","competitive::data_structure::compressed_segment_tree::CompressedSegmentTree4d"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()