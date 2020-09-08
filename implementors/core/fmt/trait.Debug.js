(function() {var implementors = {};
implementors["competitive"] = [{"text":"impl&lt;T:&nbsp;Debug + Clone + Ord + MinimumBounded&gt; Debug for MaxOperation&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Debug + Clone + Ord + MaximumBounded&gt; Debug for MinOperation&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Debug + Clone + PartialEq&gt; Debug for FirstOperation&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Debug + Clone + PartialEq&gt; Debug for LastOperation&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Debug + Copy + Zero + Add&lt;Output = T&gt;&gt; Debug for AdditiveOperation&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Debug + Copy + One + Mul&lt;Output = T&gt;&gt; Debug for MultiplicativeOperation&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Debug + Copy + PartialEq + Zero + Add&lt;Output = T&gt; + One + Mul&lt;Output = T&gt;&gt; Debug for LinearOperation&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Debug + Copy + PartialEq + BitAndIdentity&gt; Debug for BitAndOperation&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Debug + Copy + PartialEq + BitOrIdentity&gt; Debug for BitOrOperation&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Debug + Copy + PartialEq + BitXorIdentity&gt; Debug for BitXorOperation&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Debug + Clone + PartialEq, F:&nbsp;Debug + Fn(&amp;T, &amp;T) -&gt; T&gt; Debug for MonoidalOperation&lt;T, F&gt;","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Debug + Clone + PartialEq, F:&nbsp;Debug + Fn(&amp;T, &amp;T) -&gt; T, G:&nbsp;Debug + Fn(&amp;T) -&gt; T&gt; Debug for GroupOperation&lt;T, F, G&gt;","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Debug + Clone + PartialEq, F:&nbsp;Debug + Fn(&amp;T, &amp;T) -&gt; T&gt; Debug for AssocoativeOperator&lt;T, F&gt;","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Debug + Clone + PartialEq, F:&nbsp;Debug + Fn(&amp;T, &amp;T) -&gt; T&gt; Debug for AbsorbedAssocoativeOperator&lt;T, F&gt;","synthetic":false,"types":[]},{"text":"impl&lt;M1:&nbsp;Debug, M2:&nbsp;Debug&gt; Debug for CartesianOperation&lt;M1, M2&gt;","synthetic":false,"types":[]},{"text":"impl&lt;M:&nbsp;Debug&gt; Debug for CountingOperation&lt;M&gt;","synthetic":false,"types":[]},{"text":"impl&lt;M:&nbsp;Debug&gt; Debug for ReverseOperation&lt;M&gt;","synthetic":false,"types":[]},{"text":"impl Debug for CHTLine","synthetic":false,"types":[]},{"text":"impl Debug for ConvexHullTrick","synthetic":false,"types":[]},{"text":"impl&lt;M:&nbsp;Debug + Monoid&gt; Debug for BinaryIndexedTree&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M::T: Debug,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;M:&nbsp;Debug + Monoid&gt; Debug for BinaryIndexedTree2D&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M::T: Debug,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl Debug for BitSet","synthetic":false,"types":[]},{"text":"impl&lt;S:&nbsp;Debug + SemiGroup&gt; Debug for DisjointSparseTable&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S::T: Debug,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;M:&nbsp;Debug + Monoid, E:&nbsp;Debug + Monoid, F:&nbsp;Debug + Fn(&amp;M::T, &amp;E::T) -&gt; M::T&gt; Debug for LazySegmentTree&lt;M, E, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M::T: Debug,<br>&nbsp;&nbsp;&nbsp;&nbsp;E::T: Debug,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;M:&nbsp;Debug + Monoid&gt; Debug for SegmentTree&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M::T: Debug,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;M:&nbsp;Debug + Monoid&gt; Debug for QueueAggregation&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M::T: Debug,<br>&nbsp;&nbsp;&nbsp;&nbsp;M::T: Debug,<br>&nbsp;&nbsp;&nbsp;&nbsp;M::T: Debug,<br>&nbsp;&nbsp;&nbsp;&nbsp;M::T: Debug,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;M:&nbsp;Debug + Monoid&gt; Debug for DequeAggregation&lt;M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M::T: Debug,<br>&nbsp;&nbsp;&nbsp;&nbsp;M::T: Debug,<br>&nbsp;&nbsp;&nbsp;&nbsp;M::T: Debug,<br>&nbsp;&nbsp;&nbsp;&nbsp;M::T: Debug,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl Debug for UnionFind","synthetic":false,"types":[]},{"text":"impl&lt;G:&nbsp;Debug + Group&gt; Debug for WeightedUnionFind&lt;G&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;G::T: Debug,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl Debug for Circle","synthetic":false,"types":[]},{"text":"impl Debug for Line","synthetic":false,"types":[]},{"text":"impl Debug for LineSegment","synthetic":false,"types":[]},{"text":"impl Debug for Real","synthetic":false,"types":[]},{"text":"impl Debug for CCW","synthetic":false,"types":[]},{"text":"impl Debug for Adjacent","synthetic":false,"types":[]},{"text":"impl Debug for Graph","synthetic":false,"types":[]},{"text":"impl Debug for GraphRec","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; Debug for GraphEidCache&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl Debug for GridGraph","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; Debug for Adjacent4&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; Debug for Adjacent8&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl Debug for RevGraph","synthetic":false,"types":[]},{"text":"impl Debug for RevEdge","synthetic":false,"types":[]},{"text":"impl Debug for FordFulkerson","synthetic":false,"types":[]},{"text":"impl Debug for Dinic","synthetic":false,"types":[]},{"text":"impl Debug for RevCEdge","synthetic":false,"types":[]},{"text":"impl Debug for PrimalDual","synthetic":false,"types":[]},{"text":"impl Debug for StronglyConnectedComponent","synthetic":false,"types":[]},{"text":"impl Debug for TwoSatisfiability","synthetic":false,"types":[]},{"text":"impl&lt;M:&nbsp;Debug + Modulus&gt; Debug for MemorizedFactorial&lt;M&gt;","synthetic":false,"types":[]},{"text":"impl&lt;M:&nbsp;Debug + Modulus&gt; Debug for SmallModMemorizedFactorial&lt;M&gt;","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Debug&gt; Debug for Matrix&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl Debug for BabyStepGiantStep","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Debug&gt; Debug for Polynomial&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl Debug for PrimeTable","synthetic":false,"types":[]},{"text":"impl Debug for EulerPhiTable","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Debug&gt; Debug for Complex&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl&lt;M:&nbsp;Modulus&gt; Debug for MInt&lt;M&gt;","synthetic":false,"types":[]},{"text":"impl Debug for QuadDouble","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Debug + Eq&gt; Debug for KnuthMorrisPratt&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl Debug for RollingHash","synthetic":false,"types":[]},{"text":"impl Debug for MultipleRollingHash","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Debug&gt; Debug for SuffixArray&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl Debug for Zarray","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Debug + Eq + Hash&gt; Debug for Counter&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl Debug for Xorshift","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; Debug for Scanner&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl Debug for Usize1","synthetic":false,"types":[]},{"text":"impl Debug for Chars","synthetic":false,"types":[]},{"text":"impl Debug for CharsWithBase","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Debug + IterScan, B:&nbsp;Debug + FromIterator&lt;&lt;T as IterScan&gt;::Output&gt;&gt; Debug for Collect&lt;T, B&gt;","synthetic":false,"types":[]},{"text":"impl Debug for EulerTourForEdge","synthetic":false,"types":[]},{"text":"impl Debug for EulerTourForVertex","synthetic":false,"types":[]},{"text":"impl Debug for EulerTourForRichVertex","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; Debug for LowestCommonAncestor&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl Debug for LCAMonoid","synthetic":false,"types":[]},{"text":"impl&lt;M:&nbsp;Debug + Monoid, F:&nbsp;Debug + Fn(&amp;M::T, usize, Option&lt;usize&gt;) -&gt; M::T&gt; Debug for ReRooting&lt;M, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;M::T: Debug,<br>&nbsp;&nbsp;&nbsp;&nbsp;M::T: Debug,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl Debug for TreeRec","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()