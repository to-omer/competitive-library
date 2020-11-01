(function() {var implementors = {};
implementors["codesnip"] = [{"text":"impl Unpin for Config","synthetic":true,"types":[]},{"text":"impl Unpin for Opt","synthetic":true,"types":[]},{"text":"impl Unpin for Command","synthetic":true,"types":[]},{"text":"impl Unpin for VSCode","synthetic":true,"types":[]}];
implementors["codesnip_core"] = [{"text":"impl&lt;'a, 'i&gt; Unpin for Filter&lt;'a, 'i&gt;","synthetic":true,"types":[]},{"text":"impl Unpin for LinkedSnippet","synthetic":true,"types":[]},{"text":"impl Unpin for SnippetMap","synthetic":true,"types":[]},{"text":"impl Unpin for Error","synthetic":true,"types":[]},{"text":"impl Unpin for Entry","synthetic":true,"types":[]},{"text":"impl Unpin for EntryArgs","synthetic":true,"types":[]},{"text":"impl Unpin for EntryArgName","synthetic":true,"types":[]},{"text":"impl Unpin for EntryArgInclude","synthetic":true,"types":[]},{"text":"impl Unpin for EntryArgInline","synthetic":true,"types":[]},{"text":"impl Unpin for EntryArgNoInline","synthetic":true,"types":[]},{"text":"impl Unpin for EntryArg","synthetic":true,"types":[]}];
implementors["competitive"] = [{"text":"impl&lt;T&gt; Unpin for MaxOperation&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Unpin for MinOperation&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Unpin for FirstOperation&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Unpin for LastOperation&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Unpin for AdditiveOperation&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Unpin for MultiplicativeOperation&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Unpin for LinearOperation&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Unpin for BitAndOperation&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Unpin for BitOrOperation&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Unpin for BitXorOperation&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T, F&gt; Unpin for MonoidalOperation&lt;T, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;F: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;T, F, G&gt; Unpin for GroupOperation&lt;T, F, G&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;F: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;G: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;T, F&gt; Unpin for AssocoativeOperator&lt;T, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;F: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;T, F&gt; Unpin for AbsorbedAssocoativeOperator&lt;T, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;F: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M1, M2&gt; Unpin for CartesianOperation&lt;M1, M2&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M1: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;M2: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; Unpin for CountingOperation&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; Unpin for ReverseOperation&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Unpin for CHTLine","synthetic":true,"types":[]},{"text":"impl Unpin for ConvexHullTrick","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Unpin for SlideMinimum&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; Unpin for SubsetTransform&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; Unpin for SupersetTransform&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; Unpin for DivisorTransform&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; Unpin for MultipleTransform&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Unpin for KnapsackPloblemSmallWeight","synthetic":true,"types":[]},{"text":"impl Unpin for KnapsackPloblemSmallValue","synthetic":true,"types":[]},{"text":"impl Unpin for ZeroOneKnapsackProblemSmallItems","synthetic":true,"types":[]},{"text":"impl Unpin for ZeroOneKnapsackPloblemBranchAndBound","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Unpin for LongestIncreasingSubsequence&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;X, Y&gt; Unpin for IntersectionAutomaton&lt;X, Y&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;X: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;Y: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;X, Y&gt; Unpin for UnionAutomaton&lt;X, Y&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;X: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;Y: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;X, Y&gt; Unpin for ProductAutomaton&lt;X, Y&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;X: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;Y: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;'a, T&gt; Unpin for LessThanAutomaton&lt;'a, T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'a, T&gt; Unpin for GreaterThanAutomaton&lt;'a, T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'a, T&gt; Unpin for ContainAutomaton&lt;'a, T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'a, T&gt; Unpin for ContainCounterAutomaton&lt;'a, T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;A&gt; Unpin for AlwaysAcceptingAutomaton&lt;A&gt;","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; Unpin for BinaryIndexedTree&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;M as Magma&gt;::T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; Unpin for BinaryIndexedTree2D&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;M as Magma&gt;::T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Unpin for BitVector","synthetic":true,"types":[]},{"text":"impl Unpin for BitSet","synthetic":true,"types":[]},{"text":"impl&lt;S&gt; Unpin for DisjointSparseTable&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;S as Magma&gt;::T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;T, U, V&gt; Unpin for Static2DTree&lt;T, U, V&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;U: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;V: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M, E, F&gt; Unpin for LazySegmentTree&lt;M, E, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;E: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;F: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;M: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;E as Magma&gt;::T: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;M as Magma&gt;::T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; Unpin for SegmentTree&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;M as Magma&gt;::T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; Unpin for DequeAggregation&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;M as Magma&gt;::T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; Unpin for QueueAggregation&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;M as Magma&gt;::T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Unpin for Trie","synthetic":true,"types":[]},{"text":"impl Unpin for UnionFind","synthetic":true,"types":[]},{"text":"impl&lt;G&gt; Unpin for WeightedUnionFind&lt;G&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;G: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;G as Magma&gt;::T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;T, F&gt; Unpin for MergingUnionFind&lt;T, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;F: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Unpin for WaveletMatrix","synthetic":true,"types":[]},{"text":"impl Unpin for Circle","synthetic":true,"types":[]},{"text":"impl Unpin for Line","synthetic":true,"types":[]},{"text":"impl Unpin for LineSegment","synthetic":true,"types":[]},{"text":"impl Unpin for Real","synthetic":true,"types":[]},{"text":"impl Unpin for CCW","synthetic":true,"types":[]},{"text":"impl Unpin for AdjacencyListGraph","synthetic":true,"types":[]},{"text":"impl&lt;U, T&gt; Unpin for AdjacencyListGraphScanner&lt;U, T&gt;","synthetic":true,"types":[]},{"text":"impl Unpin for EdgeListGraph","synthetic":true,"types":[]},{"text":"impl&lt;U, T&gt; Unpin for EdgeListGraphScanner&lt;U, T&gt;","synthetic":true,"types":[]},{"text":"impl Unpin for GridGraph","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Unpin for LowLink&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl Unpin for DinicBuilder","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Unpin for Dinic&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl Unpin for PrimalDualBuilder","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Unpin for PrimalDual&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl Unpin for Adjacency","synthetic":true,"types":[]},{"text":"impl&lt;D&gt; Unpin for SparseGraph&lt;D&gt;","synthetic":true,"types":[]},{"text":"impl&lt;U, T&gt; Unpin for TreeGraphScanner&lt;U, T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Unpin for StronglyConnectedComponent&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl Unpin for TwoSatisfiability","synthetic":true,"types":[]},{"text":"impl Unpin for Adjacency","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Unpin for Adjacency4&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Unpin for Adjacency8&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl Unpin for DirectedEdge","synthetic":true,"types":[]},{"text":"impl Unpin for UndirectedEdge","synthetic":true,"types":[]},{"text":"impl Unpin for BidirectionalEdge","synthetic":true,"types":[]},{"text":"impl&lt;U, T, D&gt; Unpin for SparseGraphScanner&lt;U, T, D&gt;","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; Unpin for MemorizedFactorial&lt;M&gt;","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; Unpin for SmallModMemorizedFactorial&lt;M&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T, Multiplier&gt; Unpin for FormalPowerSeries&lt;T, Multiplier&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Multiplier: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; Unpin for DefaultFormalPowerSeriesMultiplier&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Unpin for Matrix&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; Unpin for NumberTheoreticTransform&lt;M&gt;","synthetic":true,"types":[]},{"text":"impl Unpin for BabyStepGiantStep","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Unpin for Polynomial&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Unpin for PrimeTable","synthetic":true,"types":[]},{"text":"impl Unpin for EulerPhiTable","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Unpin for Complex&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;M&gt; Unpin for MInt&lt;M&gt;","synthetic":true,"types":[]},{"text":"impl Unpin for QuadDouble","synthetic":true,"types":[]},{"text":"impl Unpin for Modulo1000000007","synthetic":true,"types":[]},{"text":"impl Unpin for Modulo1000000009","synthetic":true,"types":[]},{"text":"impl Unpin for Modulo998244353","synthetic":true,"types":[]},{"text":"impl Unpin for DynModulo","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Unpin for KnuthMorrisPratt&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Unpin for MultipleRollingHash","synthetic":true,"types":[]},{"text":"impl Unpin for RollingHash","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Unpin for SuffixArray&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Unpin for Zarray","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Unpin for Counter&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Unpin for SimuratedAnnealing","synthetic":true,"types":[]},{"text":"impl Unpin for Xorshift","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Unpin for Scanner&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Unpin for TotalOrd&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Unpin for Usize1","synthetic":true,"types":[]},{"text":"impl Unpin for Chars","synthetic":true,"types":[]},{"text":"impl Unpin for CharsWithBase","synthetic":true,"types":[]},{"text":"impl&lt;T, B&gt; Unpin for Collect&lt;T, B&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Unpin for EulerTourForEdge&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Unpin for EulerTourForVertex&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Unpin for EulerTourForRichVertex&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Unpin for LowestCommonAncestor&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl Unpin for LCAMonoid","synthetic":true,"types":[]},{"text":"impl Unpin for HeavyLightDecomposition","synthetic":true,"types":[]},{"text":"impl&lt;'a, M, F&gt; Unpin for ReRooting&lt;'a, M, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;F: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;M: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;M as Magma&gt;::T: Unpin,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Unpin for TreeRec","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()