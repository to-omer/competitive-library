(function() {var implementors = {};
implementors["competitive"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> for <a class=\"struct\" href=\"competitive/data_structure/struct.BitSet.html\" title=\"struct competitive::data_structure::BitSet\">BitSet</a>","synthetic":false,"types":["competitive::data_structure::bitset::BitSet"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> for <a class=\"struct\" href=\"competitive/geometry/struct.Approx.html\" title=\"struct competitive::geometry::Approx\">Approx</a>&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"competitive/geometry/trait.ApproxOrd.html\" title=\"trait competitive::geometry::ApproxOrd\">ApproxOrd</a>,&nbsp;</span>","synthetic":false,"types":["competitive::geometry::approx::Approx"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> for <a class=\"enum\" href=\"competitive/geometry/enum.Ccw.html\" title=\"enum competitive::geometry::Ccw\">Ccw</a>","synthetic":false,"types":["competitive::geometry::ccw::Ccw"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> for <a class=\"struct\" href=\"competitive/graph/struct.GridGraph.html\" title=\"struct competitive::graph::GridGraph\">GridGraph</a>","synthetic":false,"types":["competitive::graph::grid::GridGraph"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> for <a class=\"struct\" href=\"competitive/graph/struct.DirectedEdge.html\" title=\"struct competitive::graph::DirectedEdge\">DirectedEdge</a>","synthetic":false,"types":["competitive::graph::sparse_graph::DirectedEdge"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> for <a class=\"struct\" href=\"competitive/graph/struct.UndirectedEdge.html\" title=\"struct competitive::graph::UndirectedEdge\">UndirectedEdge</a>","synthetic":false,"types":["competitive::graph::sparse_graph::UndirectedEdge"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> for <a class=\"struct\" href=\"competitive/graph/struct.BidirectionalEdge.html\" title=\"struct competitive::graph::BidirectionalEdge\">BidirectionalEdge</a>","synthetic":false,"types":["competitive::graph::sparse_graph::BidirectionalEdge"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> for <a class=\"struct\" href=\"competitive/graph/struct.Adjacency.html\" title=\"struct competitive::graph::Adjacency\">Adjacency</a>","synthetic":false,"types":["competitive::graph::sparse_graph::Adjacency"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> for <a class=\"struct\" href=\"competitive/num/struct.Complex.html\" title=\"struct competitive::num::Complex\">Complex</a>&lt;T&gt;","synthetic":false,"types":["competitive::num::complex::Complex"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> for <a class=\"struct\" href=\"competitive/num/struct.DoubleDouble.html\" title=\"struct competitive::num::DoubleDouble\">DoubleDouble</a>","synthetic":false,"types":["competitive::num::double_double::DoubleDouble"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> for <a class=\"struct\" href=\"competitive/num/struct.Float32.html\" title=\"struct competitive::num::Float32\">Float32</a>","synthetic":false,"types":["competitive::num::float::Float32"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> for <a class=\"struct\" href=\"competitive/num/struct.Float64.html\" title=\"struct competitive::num::Float64\">Float64</a>","synthetic":false,"types":["competitive::num::float::Float64"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> for <a class=\"struct\" href=\"competitive/num/struct.Saturating.html\" title=\"struct competitive::num::Saturating\">Saturating</a>&lt;T&gt;","synthetic":false,"types":["competitive::num::integer::Saturating"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> for <a class=\"struct\" href=\"competitive/num/struct.Wrapping.html\" title=\"struct competitive::num::Wrapping\">Wrapping</a>&lt;T&gt;","synthetic":false,"types":["competitive::num::integer::Wrapping"]},{"text":"impl&lt;T, U&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> for <a class=\"struct\" href=\"competitive/tools/struct.PartialIgnoredOrd.html\" title=\"struct competitive::tools::PartialIgnoredOrd\">PartialIgnoredOrd</a>&lt;T, U&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>,&nbsp;</span>","synthetic":false,"types":["competitive::tools::partial_ignored_ord::PartialIgnoredOrd"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> for <a class=\"struct\" href=\"competitive/tools/struct.NotEmptySegment.html\" title=\"struct competitive::tools::NotEmptySegment\">NotEmptySegment</a>&lt;T&gt;","synthetic":false,"types":["competitive::tools::random::random_generator::NotEmptySegment"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialOrd.html\" title=\"trait core::cmp::PartialOrd\">PartialOrd</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> for <a class=\"struct\" href=\"competitive/tools/struct.TotalOrd.html\" title=\"struct competitive::tools::TotalOrd\">TotalOrd</a>&lt;T&gt;","synthetic":false,"types":["competitive::tools::totalord::TotalOrd"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()