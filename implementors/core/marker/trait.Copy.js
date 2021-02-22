(function() {var implementors = {};
implementors["competitive"] = [{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> + <a class=\"trait\" href=\"competitive/algebra/trait.MinimumBounded.html\" title=\"trait competitive::algebra::MinimumBounded\">MinimumBounded</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/algebra/struct.MaxOperation.html\" title=\"struct competitive::algebra::MaxOperation\">MaxOperation</a>&lt;T&gt;","synthetic":false,"types":["competitive::algebra::operations::MaxOperation"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> + <a class=\"trait\" href=\"competitive/algebra/trait.MaximumBounded.html\" title=\"trait competitive::algebra::MaximumBounded\">MaximumBounded</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/algebra/struct.MinOperation.html\" title=\"struct competitive::algebra::MinOperation\">MinOperation</a>&lt;T&gt;","synthetic":false,"types":["competitive::algebra::operations::MinOperation"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/algebra/struct.FirstOperation.html\" title=\"struct competitive::algebra::FirstOperation\">FirstOperation</a>&lt;T&gt;","synthetic":false,"types":["competitive::algebra::operations::FirstOperation"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/algebra/struct.LastOperation.html\" title=\"struct competitive::algebra::LastOperation\">LastOperation</a>&lt;T&gt;","synthetic":false,"types":["competitive::algebra::operations::LastOperation"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"competitive/num/trait.Zero.html\" title=\"trait competitive::num::Zero\">Zero</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;Output = T&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/algebra/struct.AdditiveOperation.html\" title=\"struct competitive::algebra::AdditiveOperation\">AdditiveOperation</a>&lt;T&gt;","synthetic":false,"types":["competitive::algebra::operations::AdditiveOperation"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"competitive/num/trait.One.html\" title=\"trait competitive::num::One\">One</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;Output = T&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/algebra/struct.MultiplicativeOperation.html\" title=\"struct competitive::algebra::MultiplicativeOperation\">MultiplicativeOperation</a>&lt;T&gt;","synthetic":false,"types":["competitive::algebra::operations::MultiplicativeOperation"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"competitive/num/trait.Zero.html\" title=\"trait competitive::num::Zero\">Zero</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;Output = T&gt; + <a class=\"trait\" href=\"competitive/num/trait.One.html\" title=\"trait competitive::num::One\">One</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;Output = T&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/algebra/struct.LinearOperation.html\" title=\"struct competitive::algebra::LinearOperation\">LinearOperation</a>&lt;T&gt;","synthetic":false,"types":["competitive::algebra::operations::LinearOperation"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"competitive/algebra/trait.BitAndIdentity.html\" title=\"trait competitive::algebra::BitAndIdentity\">BitAndIdentity</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/algebra/struct.BitAndOperation.html\" title=\"struct competitive::algebra::BitAndOperation\">BitAndOperation</a>&lt;T&gt;","synthetic":false,"types":["competitive::algebra::operations::BitAndOperation"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"competitive/algebra/trait.BitOrIdentity.html\" title=\"trait competitive::algebra::BitOrIdentity\">BitOrIdentity</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/algebra/struct.BitOrOperation.html\" title=\"struct competitive::algebra::BitOrOperation\">BitOrOperation</a>&lt;T&gt;","synthetic":false,"types":["competitive::algebra::operations::BitOrOperation"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"competitive/algebra/trait.BitXorIdentity.html\" title=\"trait competitive::algebra::BitXorIdentity\">BitXorIdentity</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/algebra/struct.BitXorOperation.html\" title=\"struct competitive::algebra::BitXorOperation\">BitXorOperation</a>&lt;T&gt;","synthetic":false,"types":["competitive::algebra::operations::BitXorOperation"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> + <a class=\"trait\" href=\"competitive/algebra/trait.MinimumBounded.html\" title=\"trait competitive::algebra::MinimumBounded\">MinimumBounded</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/algebra/struct.Top2Operation.html\" title=\"struct competitive::algebra::Top2Operation\">Top2Operation</a>&lt;T&gt;","synthetic":false,"types":["competitive::algebra::operations::Top2Operation"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/algebra/struct.PermutationOperation.html\" title=\"struct competitive::algebra::PermutationOperation\">PermutationOperation</a>","synthetic":false,"types":["competitive::algebra::operations::PermutationOperation"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"enum\" href=\"competitive/geometry/enum.CCW.html\" title=\"enum competitive::geometry::CCW\">CCW</a>","synthetic":false,"types":["competitive::geometry::CCW"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/graph/adjacency_list_graph/struct.Adjacency.html\" title=\"struct competitive::graph::adjacency_list_graph::Adjacency\">Adjacency</a>","synthetic":false,"types":["competitive::graph::adjacency_list::adjacency_list_graph::Adjacency"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/graph/grid_graph/struct.GridGraph.html\" title=\"struct competitive::graph::grid_graph::GridGraph\">GridGraph</a>","synthetic":false,"types":["competitive::graph::grid::grid_graph::GridGraph"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/graph/sparse_graph/struct.DirectedEdge.html\" title=\"struct competitive::graph::sparse_graph::DirectedEdge\">DirectedEdge</a>","synthetic":false,"types":["competitive::graph::sparse::sparse_graph::DirectedEdge"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/graph/sparse_graph/struct.UndirectedEdge.html\" title=\"struct competitive::graph::sparse_graph::UndirectedEdge\">UndirectedEdge</a>","synthetic":false,"types":["competitive::graph::sparse::sparse_graph::UndirectedEdge"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/graph/sparse_graph/struct.BidirectionalEdge.html\" title=\"struct competitive::graph::sparse_graph::BidirectionalEdge\">BidirectionalEdge</a>","synthetic":false,"types":["competitive::graph::sparse::sparse_graph::BidirectionalEdge"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/graph/sparse_graph/struct.Adjacency.html\" title=\"struct competitive::graph::sparse_graph::Adjacency\">Adjacency</a>","synthetic":false,"types":["competitive::graph::sparse::sparse_graph::Adjacency"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/num/struct.Complex.html\" title=\"struct competitive::num::Complex\">Complex</a>&lt;T&gt;","synthetic":false,"types":["competitive::num::complex::Complex"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/num/struct.Saturating.html\" title=\"struct competitive::num::Saturating\">Saturating</a>&lt;T&gt;","synthetic":false,"types":["competitive::num::integer::Saturating"]},{"text":"impl&lt;M&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/num/struct.MInt.html\" title=\"struct competitive::num::MInt\">MInt</a>&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: <a class=\"trait\" href=\"competitive/num/trait.MIntBase.html\" title=\"trait competitive::num::MIntBase\">MIntBase</a>,&nbsp;</span>","synthetic":false,"types":["competitive::num::mint::MInt"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/num/struct.QuadDouble.html\" title=\"struct competitive::num::QuadDouble\">QuadDouble</a>","synthetic":false,"types":["competitive::num::quad_double::QuadDouble"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/tools/marker/struct.Usize1.html\" title=\"struct competitive::tools::marker::Usize1\">Usize1</a>","synthetic":false,"types":["competitive::tools::scanner::marker::Usize1"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/tools/marker/struct.Chars.html\" title=\"struct competitive::tools::marker::Chars\">Chars</a>","synthetic":false,"types":["competitive::tools::scanner::marker::Chars"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/tools/marker/struct.CharsWithBase.html\" title=\"struct competitive::tools::marker::CharsWithBase\">CharsWithBase</a>","synthetic":false,"types":["competitive::tools::scanner::marker::CharsWithBase"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"competitive/tools/trait.IterScan.html\" title=\"trait competitive::tools::IterScan\">IterScan</a>, B:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/collect/trait.FromIterator.html\" title=\"trait core::iter::traits::collect::FromIterator\">FromIterator</a>&lt;&lt;T as <a class=\"trait\" href=\"competitive/tools/trait.IterScan.html\" title=\"trait competitive::tools::IterScan\">IterScan</a>&gt;::<a class=\"type\" href=\"competitive/tools/trait.IterScan.html#associatedtype.Output\" title=\"type competitive::tools::IterScan::Output\">Output</a>&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"competitive/tools/marker/struct.Collect.html\" title=\"struct competitive::tools::marker::Collect\">Collect</a>&lt;T, B&gt;","synthetic":false,"types":["competitive::tools::scanner::marker::Collect"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()