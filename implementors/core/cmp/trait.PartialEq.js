(function() {var implementors = {};
implementors["competitive"] = [{"text":"impl&lt;M:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"competitive/algebra/magma/trait.Monoid.html\" title=\"trait competitive::algebra::magma::Monoid\">Monoid</a>, T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>, F:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;</a>T, &amp;M::<a class=\"type\" href=\"competitive/algebra/magma/trait.Magma.html#associatedtype.T\" title=\"type competitive::algebra::magma::Magma::T\">T</a>) -&gt; T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/algebra/effect/struct.AnyMonoidEffect.html\" title=\"struct competitive::algebra::effect::AnyMonoidEffect\">AnyMonoidEffect</a>&lt;M, T, F&gt;&gt; for <a class=\"struct\" href=\"competitive/algebra/effect/struct.AnyMonoidEffect.html\" title=\"struct competitive::algebra::effect::AnyMonoidEffect\">AnyMonoidEffect</a>&lt;M, T, F&gt;","synthetic":false,"types":["competitive::algebra::effect::AnyMonoidEffect"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> + <a class=\"trait\" href=\"competitive/algebra/operations/trait.MinimumBounded.html\" title=\"trait competitive::algebra::operations::MinimumBounded\">MinimumBounded</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/algebra/operations/struct.MaxOperation.html\" title=\"struct competitive::algebra::operations::MaxOperation\">MaxOperation</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"competitive/algebra/operations/struct.MaxOperation.html\" title=\"struct competitive::algebra::operations::MaxOperation\">MaxOperation</a>&lt;T&gt;","synthetic":false,"types":["competitive::algebra::operations::MaxOperation"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> + <a class=\"trait\" href=\"competitive/algebra/operations/trait.MaximumBounded.html\" title=\"trait competitive::algebra::operations::MaximumBounded\">MaximumBounded</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/algebra/operations/struct.MinOperation.html\" title=\"struct competitive::algebra::operations::MinOperation\">MinOperation</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"competitive/algebra/operations/struct.MinOperation.html\" title=\"struct competitive::algebra::operations::MinOperation\">MinOperation</a>&lt;T&gt;","synthetic":false,"types":["competitive::algebra::operations::MinOperation"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/algebra/operations/struct.FirstOperation.html\" title=\"struct competitive::algebra::operations::FirstOperation\">FirstOperation</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"competitive/algebra/operations/struct.FirstOperation.html\" title=\"struct competitive::algebra::operations::FirstOperation\">FirstOperation</a>&lt;T&gt;","synthetic":false,"types":["competitive::algebra::operations::FirstOperation"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/algebra/operations/struct.LastOperation.html\" title=\"struct competitive::algebra::operations::LastOperation\">LastOperation</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"competitive/algebra/operations/struct.LastOperation.html\" title=\"struct competitive::algebra::operations::LastOperation\">LastOperation</a>&lt;T&gt;","synthetic":false,"types":["competitive::algebra::operations::LastOperation"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"competitive/algebra/operations/trait.AdditiveIdentity.html\" title=\"trait competitive::algebra::operations::AdditiveIdentity\">AdditiveIdentity</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/algebra/operations/struct.AdditiveOperation.html\" title=\"struct competitive::algebra::operations::AdditiveOperation\">AdditiveOperation</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"competitive/algebra/operations/struct.AdditiveOperation.html\" title=\"struct competitive::algebra::operations::AdditiveOperation\">AdditiveOperation</a>&lt;T&gt;","synthetic":false,"types":["competitive::algebra::operations::AdditiveOperation"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"competitive/algebra/operations/trait.MultiplicativeIdentity.html\" title=\"trait competitive::algebra::operations::MultiplicativeIdentity\">MultiplicativeIdentity</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/algebra/operations/struct.MultiplicativeOperation.html\" title=\"struct competitive::algebra::operations::MultiplicativeOperation\">MultiplicativeOperation</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"competitive/algebra/operations/struct.MultiplicativeOperation.html\" title=\"struct competitive::algebra::operations::MultiplicativeOperation\">MultiplicativeOperation</a>&lt;T&gt;","synthetic":false,"types":["competitive::algebra::operations::MultiplicativeOperation"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"competitive/algebra/operations/trait.AdditiveIdentity.html\" title=\"trait competitive::algebra::operations::AdditiveIdentity\">AdditiveIdentity</a> + <a class=\"trait\" href=\"competitive/algebra/operations/trait.MultiplicativeIdentity.html\" title=\"trait competitive::algebra::operations::MultiplicativeIdentity\">MultiplicativeIdentity</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/algebra/operations/struct.LinearOperation.html\" title=\"struct competitive::algebra::operations::LinearOperation\">LinearOperation</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"competitive/algebra/operations/struct.LinearOperation.html\" title=\"struct competitive::algebra::operations::LinearOperation\">LinearOperation</a>&lt;T&gt;","synthetic":false,"types":["competitive::algebra::operations::LinearOperation"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"competitive/algebra/operations/trait.BitAndIdentity.html\" title=\"trait competitive::algebra::operations::BitAndIdentity\">BitAndIdentity</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/algebra/operations/struct.BitAndOperation.html\" title=\"struct competitive::algebra::operations::BitAndOperation\">BitAndOperation</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"competitive/algebra/operations/struct.BitAndOperation.html\" title=\"struct competitive::algebra::operations::BitAndOperation\">BitAndOperation</a>&lt;T&gt;","synthetic":false,"types":["competitive::algebra::operations::BitAndOperation"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"competitive/algebra/operations/trait.BitOrIdentity.html\" title=\"trait competitive::algebra::operations::BitOrIdentity\">BitOrIdentity</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/algebra/operations/struct.BitOrOperation.html\" title=\"struct competitive::algebra::operations::BitOrOperation\">BitOrOperation</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"competitive/algebra/operations/struct.BitOrOperation.html\" title=\"struct competitive::algebra::operations::BitOrOperation\">BitOrOperation</a>&lt;T&gt;","synthetic":false,"types":["competitive::algebra::operations::BitOrOperation"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/algorithm/convex_hull_trick/struct.CHTLine.html\" title=\"struct competitive::algorithm::convex_hull_trick::CHTLine\">CHTLine</a>&gt; for <a class=\"struct\" href=\"competitive/algorithm/convex_hull_trick/struct.CHTLine.html\" title=\"struct competitive::algorithm::convex_hull_trick::CHTLine\">CHTLine</a>","synthetic":false,"types":["competitive::algorithm::convex_hull_trick::CHTLine"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/data_structure/bitset/struct.BitSet.html\" title=\"struct competitive::data_structure::bitset::BitSet\">BitSet</a>&gt; for <a class=\"struct\" href=\"competitive/data_structure/bitset/struct.BitSet.html\" title=\"struct competitive::data_structure::bitset::BitSet\">BitSet</a>","synthetic":false,"types":["competitive::data_structure::bitset::BitSet"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/data_structure/struct.Rev.html\" title=\"struct competitive::data_structure::Rev\">Rev</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"competitive/data_structure/struct.Rev.html\" title=\"struct competitive::data_structure::Rev\">Rev</a>&lt;T&gt;","synthetic":false,"types":["competitive::data_structure::Rev"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/data_structure/struct.TotalOrd.html\" title=\"struct competitive::data_structure::TotalOrd\">TotalOrd</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"competitive/data_structure/struct.TotalOrd.html\" title=\"struct competitive::data_structure::TotalOrd\">TotalOrd</a>&lt;T&gt;","synthetic":false,"types":["competitive::data_structure::TotalOrd"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/geometry/circle/struct.Circle.html\" title=\"struct competitive::geometry::circle::Circle\">Circle</a>&gt; for <a class=\"struct\" href=\"competitive/geometry/circle/struct.Circle.html\" title=\"struct competitive::geometry::circle::Circle\">Circle</a>","synthetic":false,"types":["competitive::geometry::circle::Circle"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/geometry/complex/struct.Complex.html\" title=\"struct competitive::geometry::complex::Complex\">Complex</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"competitive/geometry/complex/struct.Complex.html\" title=\"struct competitive::geometry::complex::Complex\">Complex</a>&lt;T&gt;","synthetic":false,"types":["competitive::geometry::complex::Complex"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/geometry/line/struct.Line.html\" title=\"struct competitive::geometry::line::Line\">Line</a>&gt; for <a class=\"struct\" href=\"competitive/geometry/line/struct.Line.html\" title=\"struct competitive::geometry::line::Line\">Line</a>","synthetic":false,"types":["competitive::geometry::line::Line"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/geometry/line/struct.LineSegment.html\" title=\"struct competitive::geometry::line::LineSegment\">LineSegment</a>&gt; for <a class=\"struct\" href=\"competitive/geometry/line/struct.LineSegment.html\" title=\"struct competitive::geometry::line::LineSegment\">LineSegment</a>","synthetic":false,"types":["competitive::geometry::line::LineSegment"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/geometry/struct.Real.html\" title=\"struct competitive::geometry::Real\">Real</a>&gt; for <a class=\"struct\" href=\"competitive/geometry/struct.Real.html\" title=\"struct competitive::geometry::Real\">Real</a>","synthetic":false,"types":["competitive::geometry::Real"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"enum\" href=\"competitive/geometry/enum.CCW.html\" title=\"enum competitive::geometry::CCW\">CCW</a>&gt; for <a class=\"enum\" href=\"competitive/geometry/enum.CCW.html\" title=\"enum competitive::geometry::CCW\">CCW</a>","synthetic":false,"types":["competitive::geometry::CCW"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/graph/graph/struct.Adjacent.html\" title=\"struct competitive::graph::graph::Adjacent\">Adjacent</a>&gt; for <a class=\"struct\" href=\"competitive/graph/graph/struct.Adjacent.html\" title=\"struct competitive::graph::graph::Adjacent\">Adjacent</a>","synthetic":false,"types":["competitive::graph::graph::Adjacent"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/math/anymod/struct.AnyMod.html\" title=\"struct competitive::math::anymod::AnyMod\">AnyMod</a>&gt; for <a class=\"struct\" href=\"competitive/math/anymod/struct.AnyMod.html\" title=\"struct competitive::math::anymod::AnyMod\">AnyMod</a>","synthetic":false,"types":["competitive::math::anymod::AnyMod"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/math/anymodu32/struct.AnyModu32.html\" title=\"struct competitive::math::anymodu32::AnyModu32\">AnyModu32</a>&gt; for <a class=\"struct\" href=\"competitive/math/anymodu32/struct.AnyModu32.html\" title=\"struct competitive::math::anymodu32::AnyModu32\">AnyModu32</a>","synthetic":false,"types":["competitive::math::anymodu32::AnyModu32"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/math/matrix/struct.Matrix.html\" title=\"struct competitive::math::matrix::Matrix\">Matrix</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"competitive/math/matrix/struct.Matrix.html\" title=\"struct competitive::math::matrix::Matrix\">Matrix</a>&lt;T&gt;","synthetic":false,"types":["competitive::math::matrix::Matrix"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/math/modi64/struct.Modi64.html\" title=\"struct competitive::math::modi64::Modi64\">Modi64</a>&gt; for <a class=\"struct\" href=\"competitive/math/modi64/struct.Modi64.html\" title=\"struct competitive::math::modi64::Modi64\">Modi64</a>","synthetic":false,"types":["competitive::math::modi64::Modi64"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/math/modu32/modulos/struct.Modulo1000000007.html\" title=\"struct competitive::math::modu32::modulos::Modulo1000000007\">Modulo1000000007</a>&gt; for <a class=\"struct\" href=\"competitive/math/modu32/modulos/struct.Modulo1000000007.html\" title=\"struct competitive::math::modu32::modulos::Modulo1000000007\">Modulo1000000007</a>","synthetic":false,"types":["competitive::math::modu32::modulos::Modulo1000000007"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/math/modu32/modulos/struct.Modulo1000000009.html\" title=\"struct competitive::math::modu32::modulos::Modulo1000000009\">Modulo1000000009</a>&gt; for <a class=\"struct\" href=\"competitive/math/modu32/modulos/struct.Modulo1000000009.html\" title=\"struct competitive::math::modu32::modulos::Modulo1000000009\">Modulo1000000009</a>","synthetic":false,"types":["competitive::math::modu32::modulos::Modulo1000000009"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/math/modu32/modulos/struct.Modulo998244353.html\" title=\"struct competitive::math::modu32::modulos::Modulo998244353\">Modulo998244353</a>&gt; for <a class=\"struct\" href=\"competitive/math/modu32/modulos/struct.Modulo998244353.html\" title=\"struct competitive::math::modu32::modulos::Modulo998244353\">Modulo998244353</a>","synthetic":false,"types":["competitive::math::modu32::modulos::Modulo998244353"]},{"text":"impl&lt;M:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/math/modu32/struct.Modu32.html\" title=\"struct competitive::math::modu32::Modu32\">Modu32</a>&lt;M&gt;&gt; for <a class=\"struct\" href=\"competitive/math/modu32/struct.Modu32.html\" title=\"struct competitive::math::modu32::Modu32\">Modu32</a>&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: <a class=\"trait\" href=\"competitive/math/modu32/trait.Modulo.html\" title=\"trait competitive::math::modu32::Modulo\">Modulo</a>,&nbsp;</span>","synthetic":false,"types":["competitive::math::modu32::Modu32"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/math/polynomial/struct.Polynomial.html\" title=\"struct competitive::math::polynomial::Polynomial\">Polynomial</a>&gt; for <a class=\"struct\" href=\"competitive/math/polynomial/struct.Polynomial.html\" title=\"struct competitive::math::polynomial::Polynomial\">Polynomial</a>","synthetic":false,"types":["competitive::math::polynomial::Polynomial"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"competitive/math/quad_double/struct.QuadDouble.html\" title=\"struct competitive::math::quad_double::QuadDouble\">QuadDouble</a>&gt; for <a class=\"struct\" href=\"competitive/math/quad_double/struct.QuadDouble.html\" title=\"struct competitive::math::quad_double::QuadDouble\">QuadDouble</a>","synthetic":false,"types":["competitive::math::quad_double::QuadDouble"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()