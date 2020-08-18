(function() {var implementors = {};
implementors["competitive"] = [{"text":"impl&lt;T&gt; UnwindSafe for MaxOperation&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; UnwindSafe for MinOperation&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; UnwindSafe for FirstOperation&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; UnwindSafe for LastOperation&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; UnwindSafe for AdditiveOperation&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; UnwindSafe for MultiplicativeOperation&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; UnwindSafe for LinearOperation&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; UnwindSafe for BitAndOperation&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; UnwindSafe for BitOrOperation&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T, F&gt; UnwindSafe for MonoidalOperation&lt;T, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;F: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;T, F, G&gt; UnwindSafe for GroupOperation&lt;T, F, G&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;F: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;G: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;T, F&gt; UnwindSafe for AssocoativeOperator&lt;T, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;F: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;T, F&gt; UnwindSafe for AbsorbedAssocoativeOperator&lt;T, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;F: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M1, M2&gt; UnwindSafe for CartesianOperation&lt;M1, M2&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M1: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;M2: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; UnwindSafe for CountingOperation&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; UnwindSafe for ReverseOperation&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; UnwindSafe for Compress&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl UnwindSafe for CHTLine","synthetic":true,"types":[]},{"text":"impl UnwindSafe for ConvexHullTrick","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; UnwindSafe for SlideMinimum&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl&lt;X, Y&gt; UnwindSafe for IntersectionAutomaton&lt;X, Y&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;X: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;Y: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;X, Y&gt; UnwindSafe for UnionAutomaton&lt;X, Y&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;X: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;Y: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;X, Y&gt; UnwindSafe for ProductAutomaton&lt;X, Y&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;X: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;Y: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;'a, T&gt; UnwindSafe for LessThanAutomaton&lt;'a, T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: RefUnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;'a, T&gt; UnwindSafe for GreaterThanAutomaton&lt;'a, T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: RefUnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;'a, T&gt; UnwindSafe for ContainAutomaton&lt;'a, T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: RefUnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;'a, T&gt; UnwindSafe for ContainCounterAutomaton&lt;'a, T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: RefUnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;A&gt; UnwindSafe for AlwaysAcceptingAutomaton&lt;A&gt;","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; UnwindSafe for BinaryIndexedTree&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;M as Magma&gt;::T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; UnwindSafe for BinaryIndexedTree2D&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;M as Magma&gt;::T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl UnwindSafe for BitVector","synthetic":true,"types":[]},{"text":"impl UnwindSafe for BitSet","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; UnwindSafe for DisjointSparseTable&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as Magma&gt;::T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;T, U, V&gt; UnwindSafe for Static2DTree&lt;T, U, V&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;U: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;V: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M, E, F&gt; UnwindSafe for LazySegmentTree&lt;M, E, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;E: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;F: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;M: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;E as Magma&gt;::T: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;M as Magma&gt;::T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; UnwindSafe for SegmentTree&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;M as Magma&gt;::T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; UnwindSafe for DequeAggregation&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;M as Magma&gt;::T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; UnwindSafe for QueueAggregation&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;M as Magma&gt;::T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Trie","synthetic":true,"types":[]},{"text":"impl UnwindSafe for UnionFind","synthetic":true,"types":[]},{"text":"impl&lt;G&gt; UnwindSafe for WeightedUnionFind&lt;G&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;G: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;G as Magma&gt;::T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;T, F&gt; UnwindSafe for MergingUnionFind&lt;T, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;F: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl UnwindSafe for WaveletMatrix","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Circle","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Line","synthetic":true,"types":[]},{"text":"impl UnwindSafe for LineSegment","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Real","synthetic":true,"types":[]},{"text":"impl UnwindSafe for CCW","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Adjacent","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Graph","synthetic":true,"types":[]},{"text":"impl&lt;U, T&gt; UnwindSafe for GraphScanner&lt;U, T&gt;","synthetic":true,"types":[]},{"text":"impl UnwindSafe for GraphRec","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; UnwindSafe for GraphEidCache&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl UnwindSafe for GridGraph","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; UnwindSafe for Adjacent4&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; UnwindSafe for Adjacent8&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl UnwindSafe for RevGraph","synthetic":true,"types":[]},{"text":"impl&lt;U, T&gt; UnwindSafe for RevGraphScanner&lt;U, T&gt;","synthetic":true,"types":[]},{"text":"impl UnwindSafe for LowLink","synthetic":true,"types":[]},{"text":"impl UnwindSafe for RevEdge","synthetic":true,"types":[]},{"text":"impl UnwindSafe for FordFulkerson","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Dinic","synthetic":true,"types":[]},{"text":"impl UnwindSafe for RevCEdge","synthetic":true,"types":[]},{"text":"impl UnwindSafe for PrimalDual","synthetic":true,"types":[]},{"text":"impl UnwindSafe for StronglyConnectedComponent","synthetic":true,"types":[]},{"text":"impl UnwindSafe for TwoSatisfiability","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; UnwindSafe for MemorizedFactorial&lt;M&gt;","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; UnwindSafe for SmallModMemorizedFactorial&lt;M&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; UnwindSafe for Matrix&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; UnwindSafe for NumberTheoreticTransform&lt;M&gt;","synthetic":true,"types":[]},{"text":"impl UnwindSafe for BabyStepGiantStep","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Polynomial","synthetic":true,"types":[]},{"text":"impl UnwindSafe for PrimeTable","synthetic":true,"types":[]},{"text":"impl UnwindSafe for EulerPhiTable","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; UnwindSafe for Complex&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; UnwindSafe for MInt&lt;M&gt;","synthetic":true,"types":[]},{"text":"impl UnwindSafe for QuadDouble","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Modulo1000000007","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Modulo1000000009","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Modulo998244353","synthetic":true,"types":[]},{"text":"impl UnwindSafe for DynModulo","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; UnwindSafe for KnuthMorrisPratt&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl UnwindSafe for MultipleRollingHash","synthetic":true,"types":[]},{"text":"impl UnwindSafe for RollingHash","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; UnwindSafe for SuffixArray&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Zarray","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; UnwindSafe for Counter&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl UnwindSafe for SimuratedAnnealing","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Xorshift","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; UnwindSafe for Scanner&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; UnwindSafe for TotalOrd&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Usize1","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Isize1","synthetic":true,"types":[]},{"text":"impl UnwindSafe for EulerTourForEdge","synthetic":true,"types":[]},{"text":"impl UnwindSafe for EulerTourForVertex","synthetic":true,"types":[]},{"text":"impl UnwindSafe for EulerTourForRichVertex","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; UnwindSafe for LowestCommonAncestor&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl UnwindSafe for LCAMonoid","synthetic":true,"types":[]},{"text":"impl UnwindSafe for HeavyLightDecomposition","synthetic":true,"types":[]},{"text":"impl&lt;M, F&gt; UnwindSafe for ReRooting&lt;M, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;F: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;M: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;M as Magma&gt;::T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl UnwindSafe for TreeRec","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()