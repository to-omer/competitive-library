(function() {var type_impls = {
"competitive":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-MarkedIterScan-for-SparseGraphScanner%3CU,+T,+D%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/competitive/graph/sparse_graph.rs.html#226-240\">source</a><a href=\"#impl-MarkedIterScan-for-SparseGraphScanner%3CU,+T,+D%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;U, T, D&gt; <a class=\"trait\" href=\"competitive/tools/trait.MarkedIterScan.html\" title=\"trait competitive::tools::MarkedIterScan\">MarkedIterScan</a> for <a class=\"struct\" href=\"competitive/graph/struct.SparseGraphScanner.html\" title=\"struct competitive::graph::SparseGraphScanner\">SparseGraphScanner</a>&lt;U, T, D&gt;<div class=\"where\">where\n    U: <a class=\"trait\" href=\"competitive/tools/trait.IterScan.html\" title=\"trait competitive::tools::IterScan\">IterScan</a>&lt;Output = <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>&gt;,\n    T: <a class=\"trait\" href=\"competitive/tools/trait.IterScan.html\" title=\"trait competitive::tools::IterScan\">IterScan</a>,\n    D: <a class=\"trait\" href=\"competitive/graph/trait.SparseGraphConstruction.html\" title=\"trait competitive::graph::SparseGraphConstruction\">SparseGraphConstruction</a>,</div></h3></section></summary><div class=\"impl-items\"><section id=\"associatedtype.Output\" class=\"associatedtype trait-impl\"><a href=\"#associatedtype.Output\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"competitive/tools/trait.MarkedIterScan.html#associatedtype.Output\" class=\"associatedtype\">Output</a> = (<a class=\"struct\" href=\"competitive/graph/struct.SparseGraph.html\" title=\"struct competitive::graph::SparseGraph\">SparseGraph</a>&lt;D&gt;, <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;&lt;T as <a class=\"trait\" href=\"competitive/tools/trait.IterScan.html\" title=\"trait competitive::tools::IterScan\">IterScan</a>&gt;::<a class=\"associatedtype\" href=\"competitive/tools/trait.IterScan.html#associatedtype.Output\" title=\"type competitive::tools::IterScan::Output\">Output</a>&gt;)</h4></section><section id=\"method.mscan\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/competitive/graph/sparse_graph.rs.html#233-239\">source</a><a href=\"#method.mscan\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"competitive/tools/trait.MarkedIterScan.html#tymethod.mscan\" class=\"fn\">mscan</a>&lt;'a, I: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html\" title=\"trait core::iter::traits::iterator::Iterator\">Iterator</a>&lt;Item = &amp;'a <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>&gt;&gt;(\n    self,\n    iter: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;mut I</a>\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;Self::<a class=\"associatedtype\" href=\"competitive/tools/trait.MarkedIterScan.html#associatedtype.Output\" title=\"type competitive::tools::MarkedIterScan::Output\">Output</a>&gt;</h4></section></div></details>","MarkedIterScan","competitive::graph::sparse_graph::DirectedGraphScanner","competitive::graph::sparse_graph::UndirectedGraphScanner","competitive::graph::sparse_graph::BidirectionalGraphScanner"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-SparseGraphScanner%3CU,+T,+D%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/competitive/graph/sparse_graph.rs.html#212-224\">source</a><a href=\"#impl-SparseGraphScanner%3CU,+T,+D%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;U, T, D&gt; <a class=\"struct\" href=\"competitive/graph/struct.SparseGraphScanner.html\" title=\"struct competitive::graph::SparseGraphScanner\">SparseGraphScanner</a>&lt;U, T, D&gt;<div class=\"where\">where\n    U: <a class=\"trait\" href=\"competitive/tools/trait.IterScan.html\" title=\"trait competitive::tools::IterScan\">IterScan</a>&lt;Output = <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>&gt;,\n    T: <a class=\"trait\" href=\"competitive/tools/trait.IterScan.html\" title=\"trait competitive::tools::IterScan\">IterScan</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.new\" class=\"method\"><a class=\"src rightside\" href=\"src/competitive/graph/sparse_graph.rs.html#217-223\">source</a><h4 class=\"code-header\">pub fn <a href=\"competitive/graph/struct.SparseGraphScanner.html#tymethod.new\" class=\"fn\">new</a>(vsize: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>, esize: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>) -&gt; Self</h4></section></summary><div class=\"docblock scraped-example-list\"><span></span><h5 id=\"scraped-examples\"><a href=\"#scraped-examples\">Examples found in repository</a><a class=\"scrape-help\" href=\"scrape-examples-help.html\">?</a></h5><div class=\"scraped-example expanded\" data-locs=\"[[[1,1],&quot;src/competitive/graph/sparse_graph.rs.html#273&quot;,&quot;line 273&quot;]]\"><div class=\"scraped-example-title\">crates/competitive/src/graph/sparse_graph.rs (<a href=\"src/competitive/graph/sparse_graph.rs.html#273\">line 273</a>)</div><div class=\"code-wrapper\"><div class=\"example-wrap\"><div data-nosnippet><pre class=\"src-line-numbers\"><span>272</span>\n<span>273</span>\n<span>274</span>\n</pre></div><pre class=\"rust\"><code>    <span class=\"kw\">fn </span>mscan&lt;<span class=\"lifetime\">'a</span>, I: Iterator&lt;Item = <span class=\"kw-2\">&amp;</span><span class=\"lifetime\">'a </span>str&gt;&gt;(<span class=\"self\">self</span>, iter: <span class=\"kw-2\">&amp;mut </span>I) -&gt; <span class=\"prelude-ty\">Option</span>&lt;<span class=\"self\">Self</span>::Output&gt; {\n        <span class=\"highlight focus\">UndirectedGraphScanner::&lt;U, T&gt;::new</span>(<span class=\"self\">self</span>.vsize, <span class=\"self\">self</span>.vsize - <span class=\"number\">1</span>).mscan(iter)\n    }</code></pre></div></div></div><details class=\"toggle more-examples-toggle\"><summary class=\"hideme\"><span>More examples</span></summary><div class=\"hide-more\">Hide additional examples</div><div class=\"more-scraped-examples\"><div class=\"toggle-line\"><div class=\"toggle-line-inner\"></div></div><div class=\"scraped-example expanded\" data-locs=\"[[[3,3],&quot;src/aizu_online_judge/grl/grl_4_a.rs.html#9&quot;,&quot;line 9&quot;]]\"><div class=\"scraped-example-title\">crates/aizu_online_judge/src/grl/grl_4_a.rs (<a href=\"src/aizu_online_judge/grl/grl_4_a.rs.html#9\">line 9</a>)</div><div class=\"code-wrapper\"><div class=\"example-wrap\"><div data-nosnippet><pre class=\"src-line-numbers\"><span>6</span>\n<span>7</span>\n<span>8</span>\n<span>9</span>\n<span>10</span>\n<span>11</span>\n</pre></div><pre class=\"rust\"><code><span class=\"kw\">pub fn </span>grl_4_a(reader: <span class=\"kw\">impl </span>Read, <span class=\"kw-2\">mut </span>writer: <span class=\"kw\">impl </span>Write) {\n    <span class=\"kw\">let </span>s = read_all_unchecked(reader);\n    <span class=\"kw\">let </span><span class=\"kw-2\">mut </span>scanner = Scanner::new(<span class=\"kw-2\">&amp;</span>s);\n    <span class=\"macro\">scan!</span>(scanner, vs, es, (graph, <span class=\"kw\">_</span>): @<span class=\"highlight focus\">DirectedGraphScanner::&lt;usize, ()&gt;::new</span>(vs, es));\n    <span class=\"macro\">writeln!</span>(writer, <span class=\"string\">\"{}\"</span>, (graph.topological_sort().len() != vs) <span class=\"kw\">as </span>u32).ok();\n}</code></pre></div></div></div><div class=\"scraped-example expanded\" data-locs=\"[[[3,3],&quot;src/aizu_online_judge/grl/grl_4_b.rs.html#9&quot;,&quot;line 9&quot;]]\"><div class=\"scraped-example-title\">crates/aizu_online_judge/src/grl/grl_4_b.rs (<a href=\"src/aizu_online_judge/grl/grl_4_b.rs.html#9\">line 9</a>)</div><div class=\"code-wrapper\"><div class=\"example-wrap\"><div data-nosnippet><pre class=\"src-line-numbers\"><span>6</span>\n<span>7</span>\n<span>8</span>\n<span>9</span>\n<span>10</span>\n<span>11</span>\n<span>12</span>\n<span>13</span>\n</pre></div><pre class=\"rust\"><code><span class=\"kw\">pub fn </span>grl_4_b(reader: <span class=\"kw\">impl </span>Read, <span class=\"kw-2\">mut </span>writer: <span class=\"kw\">impl </span>Write) {\n    <span class=\"kw\">let </span>s = read_all_unchecked(reader);\n    <span class=\"kw\">let </span><span class=\"kw-2\">mut </span>scanner = Scanner::new(<span class=\"kw-2\">&amp;</span>s);\n    <span class=\"macro\">scan!</span>(scanner, vs, es, (graph, <span class=\"kw\">_</span>): @<span class=\"highlight focus\">DirectedGraphScanner::&lt;usize, ()&gt;::new</span>(vs, es));\n    <span class=\"kw\">for </span>u <span class=\"kw\">in </span>graph.topological_sort().into_iter() {\n        <span class=\"macro\">writeln!</span>(writer, <span class=\"string\">\"{}\"</span>, u).ok();\n    }\n}</code></pre></div></div></div><div class=\"scraped-example expanded\" data-locs=\"[[[3,3],&quot;src/aizu_online_judge/grl/grl_3_b.rs.html#9&quot;,&quot;line 9&quot;]]\"><div class=\"scraped-example-title\">crates/aizu_online_judge/src/grl/grl_3_b.rs (<a href=\"src/aizu_online_judge/grl/grl_3_b.rs.html#9\">line 9</a>)</div><div class=\"code-wrapper\"><div class=\"example-wrap\"><div data-nosnippet><pre class=\"src-line-numbers\"><span>6</span>\n<span>7</span>\n<span>8</span>\n<span>9</span>\n<span>10</span>\n<span>11</span>\n<span>12</span>\n<span>13</span>\n<span>14</span>\n<span>15</span>\n</pre></div><pre class=\"rust\"><code><span class=\"kw\">pub fn </span>grl_3_b(reader: <span class=\"kw\">impl </span>Read, <span class=\"kw-2\">mut </span>writer: <span class=\"kw\">impl </span>Write) {\n    <span class=\"kw\">let </span>s = read_all_unchecked(reader);\n    <span class=\"kw\">let </span><span class=\"kw-2\">mut </span>scanner = Scanner::new(<span class=\"kw-2\">&amp;</span>s);\n    <span class=\"macro\">scan!</span>(scanner, vs, es, (graph, <span class=\"kw\">_</span>): @<span class=\"highlight focus\">UndirectedGraphScanner::&lt;usize, ()&gt;::new</span>(vs, es));\n    <span class=\"kw\">let </span><span class=\"kw-2\">mut </span>bridge = LowLink::new(<span class=\"kw-2\">&amp;</span>graph).bridge;\n    bridge.sort_unstable();\n    <span class=\"kw\">for </span>(u, v) <span class=\"kw\">in </span>bridge.into_iter() {\n        <span class=\"macro\">writeln!</span>(writer, <span class=\"string\">\"{} {}\"</span>, u, v).ok();\n    }\n}</code></pre></div></div></div><div class=\"scraped-example expanded\" data-locs=\"[[[3,3],&quot;src/aizu_online_judge/grl/grl_3_a.rs.html#9&quot;,&quot;line 9&quot;]]\"><div class=\"scraped-example-title\">crates/aizu_online_judge/src/grl/grl_3_a.rs (<a href=\"src/aizu_online_judge/grl/grl_3_a.rs.html#9\">line 9</a>)</div><div class=\"code-wrapper\"><div class=\"example-wrap\"><div data-nosnippet><pre class=\"src-line-numbers\"><span>6</span>\n<span>7</span>\n<span>8</span>\n<span>9</span>\n<span>10</span>\n<span>11</span>\n<span>12</span>\n<span>13</span>\n<span>14</span>\n<span>15</span>\n</pre></div><pre class=\"rust\"><code><span class=\"kw\">pub fn </span>grl_3_a(reader: <span class=\"kw\">impl </span>Read, <span class=\"kw-2\">mut </span>writer: <span class=\"kw\">impl </span>Write) {\n    <span class=\"kw\">let </span>s = read_all_unchecked(reader);\n    <span class=\"kw\">let </span><span class=\"kw-2\">mut </span>scanner = Scanner::new(<span class=\"kw-2\">&amp;</span>s);\n    <span class=\"macro\">scan!</span>(scanner, vs, es, (graph, <span class=\"kw\">_</span>): @<span class=\"highlight focus\">UndirectedGraphScanner::&lt;usize, ()&gt;::new</span>(vs, es));\n    <span class=\"kw\">let </span><span class=\"kw-2\">mut </span>articulation = LowLink::new(<span class=\"kw-2\">&amp;</span>graph).articulation;\n    articulation.sort_unstable();\n    <span class=\"kw\">for </span>u <span class=\"kw\">in </span>articulation.into_iter() {\n        <span class=\"macro\">writeln!</span>(writer, <span class=\"string\">\"{}\"</span>, u).ok();\n    }\n}</code></pre></div></div></div><div class=\"scraped-example expanded\" data-locs=\"[[[3,3],&quot;src/aizu_online_judge/grl/grl_3_c.rs.html#9&quot;,&quot;line 9&quot;]]\"><div class=\"scraped-example-title\">crates/aizu_online_judge/src/grl/grl_3_c.rs (<a href=\"src/aizu_online_judge/grl/grl_3_c.rs.html#9\">line 9</a>)</div><div class=\"code-wrapper\"><div class=\"example-wrap\"><div data-nosnippet><pre class=\"src-line-numbers\"><span>6</span>\n<span>7</span>\n<span>8</span>\n<span>9</span>\n<span>10</span>\n<span>11</span>\n<span>12</span>\n<span>13</span>\n<span>14</span>\n<span>15</span>\n</pre></div><pre class=\"rust\"><code><span class=\"kw\">pub fn </span>grl_3_c(reader: <span class=\"kw\">impl </span>Read, <span class=\"kw-2\">mut </span>writer: <span class=\"kw\">impl </span>Write) {\n    <span class=\"kw\">let </span>s = read_all_unchecked(reader);\n    <span class=\"kw\">let </span><span class=\"kw-2\">mut </span>scanner = Scanner::new(<span class=\"kw-2\">&amp;</span>s);\n    <span class=\"macro\">scan!</span>(scanner, vs, es, (graph, <span class=\"kw\">_</span>): @<span class=\"highlight focus\">DirectedGraphScanner::&lt;usize, ()&gt;::new</span>(vs, es));\n    <span class=\"kw\">let </span>scc = StronglyConnectedComponent::new(<span class=\"kw-2\">&amp;</span>graph);\n    <span class=\"macro\">scan!</span>(scanner, q);\n    <span class=\"kw\">for </span>(u, v) <span class=\"kw\">in </span>scanner.iter::&lt;(usize, usize)&gt;().take(q) {\n        <span class=\"macro\">writeln!</span>(writer, <span class=\"string\">\"{}\"</span>, (scc[u] == scc[v]) <span class=\"kw\">as </span>u32).ok();\n    }\n}</code></pre></div></div></div><div class=\"example-links\">Additional examples can be found in:<br><ul><li><a href=\"src/aizu_online_judge/grl/grl_1_a.rs.html#13\">crates/aizu_online_judge/src/grl/grl_1_a.rs</a></li><li><a href=\"src/aizu_online_judge/grl/grl_1_b.rs.html#12\">crates/aizu_online_judge/src/grl/grl_1_b.rs</a></li><li><a href=\"src/aizu_online_judge/grl/grl_1_c.rs.html#13\">crates/aizu_online_judge/src/grl/grl_1_c.rs</a></li></ul></div></div></details></div></details></div></details>",0,"competitive::graph::sparse_graph::DirectedGraphScanner","competitive::graph::sparse_graph::UndirectedGraphScanner","competitive::graph::sparse_graph::BidirectionalGraphScanner"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()