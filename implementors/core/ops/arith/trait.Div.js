(function() {var implementors = {};
implementors["competitive"] = [{"text":"impl&lt;T, Multiplier&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Div.html\" title=\"trait core::ops::arith::Div\">Div</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;'_ </a>T&gt; for <a class=\"struct\" href=\"competitive/math/struct.FormalPowerSeries.html\" title=\"struct competitive::math::FormalPowerSeries\">FormalPowerSeries</a>&lt;T, Multiplier&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"competitive/math/trait.FormalPowerSeriesCoefficient.html\" title=\"trait competitive::math::FormalPowerSeriesCoefficient\">FormalPowerSeriesCoefficient</a>,&nbsp;</span>","synthetic":false,"types":["competitive::math::formal_power_series::FormalPowerSeries"]},{"text":"impl&lt;T, Multiplier&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Div.html\" title=\"trait core::ops::arith::Div\">Div</a>&lt;<a class=\"struct\" href=\"competitive/math/struct.FormalPowerSeries.html\" title=\"struct competitive::math::FormalPowerSeries\">FormalPowerSeries</a>&lt;T, Multiplier&gt;&gt; for <a class=\"struct\" href=\"competitive/math/struct.FormalPowerSeries.html\" title=\"struct competitive::math::FormalPowerSeries\">FormalPowerSeries</a>&lt;T, Multiplier&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"competitive/math/trait.FormalPowerSeriesCoefficient.html\" title=\"trait competitive::math::FormalPowerSeriesCoefficient\">FormalPowerSeriesCoefficient</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Multiplier: <a class=\"trait\" href=\"competitive/math/trait.FormalPowerSeriesMultiplier.html\" title=\"trait competitive::math::FormalPowerSeriesMultiplier\">FormalPowerSeriesMultiplier</a>&lt;T = T&gt;,&nbsp;</span>","synthetic":false,"types":["competitive::math::formal_power_series::FormalPowerSeries"]},{"text":"impl&lt;T, Multiplier&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Div.html\" title=\"trait core::ops::arith::Div\">Div</a>&lt;&amp;'_ <a class=\"struct\" href=\"competitive/math/struct.FormalPowerSeries.html\" title=\"struct competitive::math::FormalPowerSeries\">FormalPowerSeries</a>&lt;T, Multiplier&gt;&gt; for &amp;<a class=\"struct\" href=\"competitive/math/struct.FormalPowerSeries.html\" title=\"struct competitive::math::FormalPowerSeries\">FormalPowerSeries</a>&lt;T, Multiplier&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"competitive/math/trait.FormalPowerSeriesCoefficient.html\" title=\"trait competitive::math::FormalPowerSeriesCoefficient\">FormalPowerSeriesCoefficient</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Multiplier: <a class=\"trait\" href=\"competitive/math/trait.FormalPowerSeriesMultiplier.html\" title=\"trait competitive::math::FormalPowerSeriesMultiplier\">FormalPowerSeriesMultiplier</a>&lt;T = T&gt;,&nbsp;</span>","synthetic":false,"types":["competitive::math::formal_power_series::FormalPowerSeries"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"competitive/num/trait.Zero.html\" title=\"trait competitive::num::Zero\">Zero</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Sub.html\" title=\"trait core::ops::arith::Sub\">Sub</a>&lt;Output = T&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;Output = T&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Div.html\" title=\"trait core::ops::arith::Div\">Div</a>&lt;Output = T&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Div.html\" title=\"trait core::ops::arith::Div\">Div</a>&lt;&amp;'_ <a class=\"struct\" href=\"competitive/math/struct.Polynomial.html\" title=\"struct competitive::math::Polynomial\">Polynomial</a>&lt;T&gt;&gt; for &amp;<a class=\"struct\" href=\"competitive/math/struct.Polynomial.html\" title=\"struct competitive::math::Polynomial\">Polynomial</a>&lt;T&gt;","synthetic":false,"types":["competitive::math::polynomial::Polynomial"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;Output = T&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Sub.html\" title=\"trait core::ops::arith::Sub\">Sub</a>&lt;Output = T&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Mul.html\" title=\"trait core::ops::arith::Mul\">Mul</a>&lt;Output = T&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Div.html\" title=\"trait core::ops::arith::Div\">Div</a>&lt;Output = T&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Div.html\" title=\"trait core::ops::arith::Div\">Div</a>&lt;<a class=\"struct\" href=\"competitive/num/struct.Complex.html\" title=\"struct competitive::num::Complex\">Complex</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"competitive/num/struct.Complex.html\" title=\"struct competitive::num::Complex\">Complex</a>&lt;T&gt;","synthetic":false,"types":["competitive::num::complex::Complex"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Div.html\" title=\"trait core::ops::arith::Div\">Div</a>&lt;Output = T&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Div.html\" title=\"trait core::ops::arith::Div\">Div</a>&lt;T&gt; for <a class=\"struct\" href=\"competitive/num/struct.Complex.html\" title=\"struct competitive::num::Complex\">Complex</a>&lt;T&gt;","synthetic":false,"types":["competitive::num::complex::Complex"]},{"text":"impl&lt;M&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Div.html\" title=\"trait core::ops::arith::Div\">Div</a>&lt;<a class=\"struct\" href=\"competitive/num/struct.MInt.html\" title=\"struct competitive::num::MInt\">MInt</a>&lt;M&gt;&gt; for <a class=\"struct\" href=\"competitive/num/struct.MInt.html\" title=\"struct competitive::num::MInt\">MInt</a>&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: <a class=\"trait\" href=\"competitive/num/trait.MIntBase.html\" title=\"trait competitive::num::MIntBase\">MIntBase</a>,&nbsp;</span>","synthetic":false,"types":["competitive::num::mint::MInt"]},{"text":"impl&lt;M&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Div.html\" title=\"trait core::ops::arith::Div\">Div</a>&lt;<a class=\"struct\" href=\"competitive/num/struct.MInt.html\" title=\"struct competitive::num::MInt\">MInt</a>&lt;M&gt;&gt; for &amp;<a class=\"struct\" href=\"competitive/num/struct.MInt.html\" title=\"struct competitive::num::MInt\">MInt</a>&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: <a class=\"trait\" href=\"competitive/num/trait.MIntBase.html\" title=\"trait competitive::num::MIntBase\">MIntBase</a>,&nbsp;</span>","synthetic":false,"types":["competitive::num::mint::MInt"]},{"text":"impl&lt;M&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Div.html\" title=\"trait core::ops::arith::Div\">Div</a>&lt;&amp;'_ <a class=\"struct\" href=\"competitive/num/struct.MInt.html\" title=\"struct competitive::num::MInt\">MInt</a>&lt;M&gt;&gt; for <a class=\"struct\" href=\"competitive/num/struct.MInt.html\" title=\"struct competitive::num::MInt\">MInt</a>&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: <a class=\"trait\" href=\"competitive/num/trait.MIntBase.html\" title=\"trait competitive::num::MIntBase\">MIntBase</a>,&nbsp;</span>","synthetic":false,"types":["competitive::num::mint::MInt"]},{"text":"impl&lt;M&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Div.html\" title=\"trait core::ops::arith::Div\">Div</a>&lt;&amp;'_ <a class=\"struct\" href=\"competitive/num/struct.MInt.html\" title=\"struct competitive::num::MInt\">MInt</a>&lt;M&gt;&gt; for &amp;<a class=\"struct\" href=\"competitive/num/struct.MInt.html\" title=\"struct competitive::num::MInt\">MInt</a>&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: <a class=\"trait\" href=\"competitive/num/trait.MIntBase.html\" title=\"trait competitive::num::MIntBase\">MIntBase</a>,&nbsp;</span>","synthetic":false,"types":["competitive::num::mint::MInt"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/arith/trait.Div.html\" title=\"trait core::ops::arith::Div\">Div</a>&lt;<a class=\"struct\" href=\"competitive/num/struct.QuadDouble.html\" title=\"struct competitive::num::QuadDouble\">QuadDouble</a>&gt; for <a class=\"struct\" href=\"competitive/num/struct.QuadDouble.html\" title=\"struct competitive::num::QuadDouble\">QuadDouble</a>","synthetic":false,"types":["competitive::num::quad_double::QuadDouble"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()