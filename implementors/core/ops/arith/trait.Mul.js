(function() {var implementors = {};
implementors["competitive"] = [{"text":"impl&lt;'a, T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;Output = T&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Sub.html\" title=\"trait core::ops::arith::Sub\">Sub</a>&lt;Output = T&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;Output = T&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;&amp;'a <a class=\"struct\" href=\"competitive/math/matrix/struct.Matrix.html\" title=\"struct competitive::math::matrix::Matrix\">Matrix</a>&lt;T&gt;&gt; for &amp;'a <a class=\"struct\" href=\"competitive/math/matrix/struct.Matrix.html\" title=\"struct competitive::math::matrix::Matrix\">Matrix</a>&lt;T&gt;","synthetic":false,"types":["competitive::math::matrix::Matrix"]},{"text":"impl&lt;'_, '_&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;&amp;'_ <a class=\"struct\" href=\"competitive/math/polynomial/struct.Polynomial.html\" title=\"struct competitive::math::polynomial::Polynomial\">Polynomial</a>&gt; for &amp;'_ <a class=\"struct\" href=\"competitive/math/polynomial/struct.Polynomial.html\" title=\"struct competitive::math::polynomial::Polynomial\">Polynomial</a>","synthetic":false,"types":["competitive::math::polynomial::Polynomial"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;Output = T&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Sub.html\" title=\"trait core::ops::arith::Sub\">Sub</a>&lt;Output = T&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;Output = T&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;<a class=\"struct\" href=\"competitive/num/complex/struct.Complex.html\" title=\"struct competitive::num::complex::Complex\">Complex</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"competitive/num/complex/struct.Complex.html\" title=\"struct competitive::num::complex::Complex\">Complex</a>&lt;T&gt;","synthetic":false,"types":["competitive::num::complex::Complex"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;Output = T&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;T&gt; for <a class=\"struct\" href=\"competitive/num/complex/struct.Complex.html\" title=\"struct competitive::num::complex::Complex\">Complex</a>&lt;T&gt;","synthetic":false,"types":["competitive::num::complex::Complex"]},{"text":"impl&lt;M:&nbsp;<a class=\"trait\" href=\"competitive/num/mint/trait.Modulus.html\" title=\"trait competitive::num::mint::Modulus\">Modulus</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;<a class=\"struct\" href=\"competitive/num/mint/struct.MInt.html\" title=\"struct competitive::num::mint::MInt\">MInt</a>&lt;M&gt;&gt; for <a class=\"struct\" href=\"competitive/num/mint/struct.MInt.html\" title=\"struct competitive::num::mint::MInt\">MInt</a>&lt;M&gt;","synthetic":false,"types":["competitive::num::mint::MInt"]},{"text":"impl&lt;'_, M:&nbsp;<a class=\"trait\" href=\"competitive/num/mint/trait.Modulus.html\" title=\"trait competitive::num::mint::Modulus\">Modulus</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;<a class=\"struct\" href=\"competitive/num/mint/struct.MInt.html\" title=\"struct competitive::num::mint::MInt\">MInt</a>&lt;M&gt;&gt; for &amp;'_ <a class=\"struct\" href=\"competitive/num/mint/struct.MInt.html\" title=\"struct competitive::num::mint::MInt\">MInt</a>&lt;M&gt;","synthetic":false,"types":["competitive::num::mint::MInt"]},{"text":"impl&lt;'_, M:&nbsp;<a class=\"trait\" href=\"competitive/num/mint/trait.Modulus.html\" title=\"trait competitive::num::mint::Modulus\">Modulus</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;&amp;'_ <a class=\"struct\" href=\"competitive/num/mint/struct.MInt.html\" title=\"struct competitive::num::mint::MInt\">MInt</a>&lt;M&gt;&gt; for <a class=\"struct\" href=\"competitive/num/mint/struct.MInt.html\" title=\"struct competitive::num::mint::MInt\">MInt</a>&lt;M&gt;","synthetic":false,"types":["competitive::num::mint::MInt"]},{"text":"impl&lt;'_, '_, M:&nbsp;<a class=\"trait\" href=\"competitive/num/mint/trait.Modulus.html\" title=\"trait competitive::num::mint::Modulus\">Modulus</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;&amp;'_ <a class=\"struct\" href=\"competitive/num/mint/struct.MInt.html\" title=\"struct competitive::num::mint::MInt\">MInt</a>&lt;M&gt;&gt; for &amp;'_ <a class=\"struct\" href=\"competitive/num/mint/struct.MInt.html\" title=\"struct competitive::num::mint::MInt\">MInt</a>&lt;M&gt;","synthetic":false,"types":["competitive::num::mint::MInt"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.f64.html\">f64</a>&gt; for <a class=\"struct\" href=\"competitive/num/quad_double/struct.QuadDouble.html\" title=\"struct competitive::num::quad_double::QuadDouble\">QuadDouble</a>","synthetic":false,"types":["competitive::num::quad_double::QuadDouble"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;<a class=\"struct\" href=\"competitive/num/quad_double/struct.QuadDouble.html\" title=\"struct competitive::num::quad_double::QuadDouble\">QuadDouble</a>&gt; for <a class=\"struct\" href=\"competitive/num/quad_double/struct.QuadDouble.html\" title=\"struct competitive::num::quad_double::QuadDouble\">QuadDouble</a>","synthetic":false,"types":["competitive::num::quad_double::QuadDouble"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()