// codesnip-guard: MethodBenchAll
#[cfg_attr(any(),rustfmt::skip)]pub fn method_bench_all(){}
// codesnip-guard: AdditiveOperation
#[cfg_attr(any(),rustfmt::skip)]pub use self::additive_operation_impl::AdditiveOperation;#[cfg_attr(any(),rustfmt::skip)]mod additive_operation_impl{use super::*;use std::{marker::PhantomData,ops::{Add,Neg,Sub}};#[doc=" $+$"]pub struct AdditiveOperation<T>where T:Clone+Zero+Add<Output=T>{_marker:PhantomData<fn()->T>}impl<T>Magma for AdditiveOperation<T>where T:Clone+Zero+Add<Output=T>{type T=T;#[inline]fn operate(x:&Self::T,y:&Self::T)->Self::T{x.clone()+y.clone()}}impl<T>Unital for AdditiveOperation<T>where T:Clone+Zero+Add<Output=T>{#[inline]fn unit()->Self::T{Zero::zero()}}impl<T>Associative for AdditiveOperation<T>where T:Clone+Zero+Add<Output=T>{}impl<T>Commutative for AdditiveOperation<T>where T:Clone+Zero+Add<Output=T>{}impl<T>Invertible for AdditiveOperation<T>where T:Clone+Zero+Add<Output=T>+Sub<Output=T>+Neg<Output=T>{#[inline]fn inverse(x:&Self::T)->Self::T{-x.clone()}#[inline]fn rinv_operate(x:&Self::T,y:&Self::T)->Self::T{x.clone()-y.clone()}}}
// codesnip-guard: BinaryIndexedTree
#[cfg_attr(any(),rustfmt::skip)]pub use self::binary_indexed_tree::BinaryIndexedTree;#[cfg_attr(any(),rustfmt::skip)]mod binary_indexed_tree{use super::{Group,Monoid};use std::fmt::{self,Debug,Formatter};pub struct BinaryIndexedTree<M>where M:Monoid{n:usize,bit:Vec<M::T>}impl<M>Clone for BinaryIndexedTree<M>where M:Monoid{fn clone(&self)->Self{Self{n:self.n,bit:self.bit.clone()}}}impl<M>Debug for BinaryIndexedTree<M>where M:Monoid<T:Debug>{fn fmt(&self,f:&mut Formatter<'_>)->fmt::Result{f.debug_struct("BinaryIndexedTree").field("n",&self.n).field("bit",&self.bit).finish()}}impl<M>BinaryIndexedTree<M>where M:Monoid{#[inline]pub fn new(n:usize)->Self{let bit=vec![M::unit();n+1];Self{n,bit}}#[inline]pub fn from_slice(slice:&[M::T])->Self{let n=slice.len();let mut bit=vec![M::unit();n+1];for(i,x)in slice.iter().enumerate(){let k=i+1;M::operate_assign(&mut bit[k],x);let j=k+(k&(!k+1));if j<=n{bit[j]=M::operate(&bit[j],&bit[k]);}}Self{n,bit}}#[inline]#[doc=" fold [0, k)"]pub fn accumulate0(&self,mut k:usize)->M::T{debug_assert!(k<=self.n);let mut res=M::unit();while k>0{res=M::operate(&res,&self.bit[k]);k-=k&(!k+1);}res}#[inline]#[doc=" fold [0, k]"]pub fn accumulate(&self,k:usize)->M::T{self.accumulate0(k+1)}#[inline]pub fn update(&mut self,k:usize,x:M::T){debug_assert!(k<self.n);let mut k=k+1;while k<=self.n{self.bit[k]=M::operate(&self.bit[k],&x);k+=k&(!k+1);}}#[inline]pub fn partition_point_acc<P>(&self,mut pred:P)->usize where P:FnMut(&M::T)->bool{let n=self.n;let mut acc=M::unit();let mut pos=0;let mut k=n.next_power_of_two();while k>0{if k+pos<=n{let nacc=M::operate(&acc,&self.bit[k+pos]);if pred(&nacc){pos+=k;acc=nacc;}}k>>=1;}pos}}impl<G:Group>BinaryIndexedTree<G>{#[inline]pub fn fold(&self,l:usize,r:usize)->G::T{debug_assert!(l<=self.n&&r<=self.n);G::operate(&G::inverse(&self.accumulate0(l)),&self.accumulate0(r))}#[inline]pub fn get(&self,k:usize)->G::T{self.fold(k,k+1)}#[inline]pub fn set(&mut self,k:usize,x:G::T){self.update(k,G::operate(&G::inverse(&self.get(k)),&x));}}}
// codesnip-guard: BitVector
#[cfg_attr(any(),rustfmt::skip)]pub use self::bit_vector::{BitVector,RankSelectDictionaries};#[cfg_attr(any(),rustfmt::skip)]mod bit_vector{use std::iter::FromIterator;#[doc=" rank_i(select_i(k)) = k"]#[doc=" rank_i(select_i(k) + 1) = k + 1"]pub trait RankSelectDictionaries{fn bit_length(&self)->usize;#[doc=" get k-th bit"]fn access(&self,k:usize)->bool;#[doc=" Returns the k-th bit and the number of ones before it."]fn access_rank1(&self,k:usize)->(bool,usize){(self.access(k),self.rank1(k))}#[doc=" the number of 1 in [0, k)"]fn rank1(&self,k:usize)->usize{(0..k).filter(|&i|self.access(i)).count()}#[doc=" the number of 0 in [0, k)"]fn rank0(&self,k:usize)->usize{k-self.rank1(k)}#[doc=" index of k-th 1"]fn select1(&self,k:usize)->Option<usize>{let n=self.bit_length();if self.rank1(n)<=k{return None;}let(mut l,mut r)=(0,n);while r-l>1{let m=l.midpoint(r);if self.rank1(m)<=k{l=m;}else{r=m;}}Some(l)}#[doc=" index of k-th 0"]fn select0(&self,k:usize)->Option<usize>{let n=self.bit_length();if self.rank0(n)<=k{return None;}let(mut l,mut r)=(0,n);while r-l>1{let m=l.midpoint(r);if self.rank0(m)<=k{l=m;}else{r=m;}}Some(l)}}macro_rules!impl_rank_select_for_bits{($($t:ty)*)=>{$(impl RankSelectDictionaries for$t{fn bit_length(&self)->usize{const WORD_SIZE:usize=(0 as$t).count_zeros()as usize;WORD_SIZE}fn access(&self,k:usize)->bool{const WORD_SIZE:usize=(0 as$t).count_zeros()as usize;if k<WORD_SIZE{self&(1 as$t)<<k!=0}else{false}}fn rank1(&self,k:usize)->usize{const WORD_SIZE:usize=(0 as$t).count_zeros()as usize;if k<WORD_SIZE{(self&!(!(0 as$t)<<k)).count_ones()as usize}else{self.count_ones()as usize}}})*};}impl_rank_select_for_bits!(u8 u16 u32 u64 usize i8 i16 i32 i64 isize u128 i128);fn select_word_scalar(mut word:u64,mut rank:usize)->usize{let count=word.count_ones()as usize;debug_assert!(rank<count);if rank<4{for _ in 0..rank{word&=word-1;}return word.trailing_zeros()as usize;}if count-rank<=4{for _ in 0..count-rank-1{word&=!(1<<(u64::BITS as usize-1-word.leading_zeros()as usize));}return u64::BITS as usize-1-word.leading_zeros()as usize;}let mut offset=0;let mut width=u64::BITS as usize/2;while width!=0{let mask=u64::MAX>>(u64::BITS as usize-width);let count=(word&mask).count_ones()as usize;if rank<count{word&=mask;}else{word>>=width;rank-=count;offset+=width;}width/=2;}offset}#[cfg(target_arch="x86_64")]#[allow(unsafe_op_in_unsafe_fn)]mod simd{use std::arch::x86_64::_pdep_u64;#[target_feature(enable="bmi2")]pub unsafe fn select_word(word:u64,rank:usize)->usize{_pdep_u64(1<<rank,word).trailing_zeros()as usize}}#[doc=" An append-only rank/select dictionary with 256-word prefix blocks."]#[doc=""]#[doc=" The layout reduces metadata and improves large or select-heavy workloads. A compact"]#[doc=" rank-only workload can be faster with an absolute prefix stored beside every word."]#[derive(Debug,Clone)]pub struct BitVector{words:Vec<u64>,super_prefix:Vec<usize>,sub_prefix:Vec<u16>,sum:usize,len:usize,#[cfg(target_arch="x86_64")]bmi2:bool}impl BitVector{const WORD_SIZE:usize=u64::BITS as usize;const SUPER_WORDS:usize=256;pub fn benchmark_scalar(self)->Self{#[cfg(target_arch="x86_64")]{let mut this=self;this.bmi2=false;return this;}#[cfg(not(target_arch="x86_64"))]self}pub fn benchmark_bmi2(self)->Self{#[cfg(target_arch="x86_64")]{assert!(is_x86_feature_detected!("bmi2"));let mut this=self;this.bmi2=true;return this;}#[cfg(not(target_arch="x86_64"))]panic!("BMI2 is unavailable");}pub fn with_capacity(bits:usize)->Self{let words=bits.div_ceil(Self::WORD_SIZE);let mut word_values=Vec::with_capacity(words+1);word_values.push(0);let mut super_prefix=Vec::with_capacity(words/Self::SUPER_WORDS+1);super_prefix.push(0);let mut sub_prefix=Vec::with_capacity(words+1);sub_prefix.push(0);Self{words:word_values,super_prefix,sub_prefix,sum:0,len:0,#[cfg(target_arch="x86_64")]bmi2:is_x86_feature_detected!("bmi2")}}pub fn push(&mut self,bit:bool){let word=self.len/Self::WORD_SIZE;let offset=self.len%Self::WORD_SIZE;if offset==0{self.words.push(0);self.sub_prefix.push(0);}if bit{self.words[word]|=1u64<<offset;self.sum+=1;}self.len+=1;if self.len.is_multiple_of(Self::WORD_SIZE){let word=self.len/Self::WORD_SIZE;if word.is_multiple_of(Self::SUPER_WORDS){if let Some(prefix)=self.super_prefix.get_mut(word/Self::SUPER_WORDS){*prefix=self.sum;}else{self.super_prefix.push(self.sum);}}self.sub_prefix[word]=(self.sum-self.super_prefix[word/Self::SUPER_WORDS])as u16;}}fn from_words(mut words:Vec<u64>,len:usize)->Self{let mut super_prefix=Vec::with_capacity(words.len().div_ceil(Self::SUPER_WORDS));let mut sub_prefix=Vec::with_capacity(words.len()+1);let mut sum=0;let mut super_sum=0;for index in 0..=words.len(){if index.is_multiple_of(Self::SUPER_WORDS){if index<words.len()||len.is_multiple_of(Self::WORD_SIZE){super_sum=sum;super_prefix.push(sum);}sub_prefix.push(0);}else{sub_prefix.push((sum-super_sum)as u16);}if let Some(&word)=words.get(index){sum+=word.count_ones()as usize;}}words.push(0);Self{words,super_prefix,sub_prefix,sum,len,#[cfg(target_arch="x86_64")]bmi2:is_x86_feature_detected!("bmi2")}}#[inline]fn select_word(&self,word:u64,rank:usize)->usize{#[cfg(target_arch="x86_64")]if self.bmi2{return unsafe{simd::select_word(word,rank)};}select_word_scalar(word,rank)}#[inline]fn locate_one(&self,mut rank:usize)->(usize,usize){let block=self.super_prefix.partition_point(|&sum|sum<=rank)-1;rank-=self.super_prefix[block];let word_start=block*Self::SUPER_WORDS;let word_end=(word_start+Self::SUPER_WORDS).min(self.words.len()-1);let lane=self.sub_prefix[word_start..word_end].partition_point(|&sum|sum as usize<=rank)-1;let word=word_start+lane;rank-=self.sub_prefix[word]as usize;(word,rank)}#[inline]fn locate_zero(&self,mut rank:usize)->(usize,usize){let mut block=0;let mut right=self.super_prefix.len();while right-block>1{let middle=block.midpoint(right);if middle*Self::SUPER_WORDS*Self::WORD_SIZE-self.super_prefix[middle]<=rank{block=middle;}else{right=middle;}}rank-=block*Self::SUPER_WORDS*Self::WORD_SIZE-self.super_prefix[block];let word_start=block*Self::SUPER_WORDS;let word_end=(word_start+Self::SUPER_WORDS).min(self.words.len()-1);let mut word=word_start;let mut right=word_end;while right-word>1{let middle=word.midpoint(right);let zeros=(middle-word_start)*Self::WORD_SIZE-self.sub_prefix[middle]as usize;if zeros<=rank{word=middle;}else{right=middle;}}rank-=(word-word_start)*Self::WORD_SIZE-self.sub_prefix[word]as usize;(word,rank)}}impl RankSelectDictionaries for BitVector{fn bit_length(&self)->usize{self.len}fn access(&self,k:usize)->bool{debug_assert!(k<self.len);self.words[k/Self::WORD_SIZE]&(1u64<<(k%Self::WORD_SIZE))!=0}fn access_rank1(&self,k:usize)->(bool,usize){debug_assert!(k<=self.len);let word=k/Self::WORD_SIZE;let offset=k%Self::WORD_SIZE;let bits=self.words[word];(bits&(1u64<<offset)!=0,self.super_prefix[word/Self::SUPER_WORDS]+self.sub_prefix[word]as usize+(bits&!(u64::MAX<<offset)).count_ones()as usize)}fn rank1(&self,k:usize)->usize{self.access_rank1(k).1}fn select1(&self,k:usize)->Option<usize>{if self.sum<=k{return None;}let(word,rank)=self.locate_one(k);Some(word*Self::WORD_SIZE+self.select_word(self.words[word],rank))}fn select0(&self,k:usize)->Option<usize>{if self.len-self.sum<=k{return None;}let(word,rank)=self.locate_zero(k);let mut bits=!self.words[word];if word+1==self.words.len()-1&&!self.len.is_multiple_of(Self::WORD_SIZE){bits&=u64::MAX>>(Self::WORD_SIZE-self.len%Self::WORD_SIZE);}Some(word*Self::WORD_SIZE+self.select_word(bits,rank))}}impl FromIterator<bool>for BitVector{fn from_iter<T:IntoIterator<Item=bool>>(iter:T)->Self{let iter=iter.into_iter();let(lower,upper)=iter.size_hint();let capacity=match upper{Some(upper)=>upper,None=>lower,};let mut words=Vec::with_capacity(capacity.div_ceil(Self::WORD_SIZE)+1);let mut word=0u64;let mut word_len=0;let mut len=0;for bit in iter{word|=(bit as u64)<<word_len;word_len+=1;len+=1;if word_len==Self::WORD_SIZE{words.push(word);word=0;word_len=0;}}if word_len!=0{words.push(word);}Self::from_words(words,len)}}}
// codesnip-guard: BucketQueue
#[cfg_attr(any(),rustfmt::skip)]pub use self::bucket_queue::{BucketQueueI8,BucketQueueI16,BucketQueueU8,BucketQueueU16};#[cfg_attr(any(),rustfmt::skip)]mod bucket_queue{#[derive(Clone,Debug)]struct BucketQueue8{counts:[u32;1<<8],occupied:[u64;1<<2],summary:u8,maximum:u8,len:usize}impl BucketQueue8{fn new()->Self{Self{counts:[0;1<<8],occupied:[0;1<<2],summary:0,maximum:0,len:0}}#[inline]fn push(&mut self,value:u8){assert!(self.len<u32::MAX as usize);let value=value as usize;if self.len==0||value>self.maximum as usize{self.maximum=value as u8;}if self.counts[value]==0{self.occupied[value/64]|=1<<(value%64);self.summary|=1<<(value/64);}self.counts[value]+=1;self.len+=1;}fn from_values(values:impl IntoIterator<Item=u8>,len:usize)->Self{assert!(len<=u32::MAX as usize);let mut result=Self::new();result.len=len;for value in values{result.counts[value as usize]+=1;}for(value,&count)in result.counts.iter().enumerate(){if count!=0{result.occupied[value/64]|=1<<(value%64);}}for(word,&occupied)in result.occupied.iter().enumerate(){if occupied!=0{result.summary|=1<<word;}}if len!=0{let word=(u8::BITS-1-result.summary.leading_zeros())as usize;result.maximum=(word*64+63-result.occupied[word].leading_zeros()as usize)as u8;}result}#[inline]fn pop(&mut self)->Option<u8>{if self.len==0{return None;}let value=self.maximum as usize;self.counts[value]-=1;self.len-=1;if self.counts[value]==0{let word=value/64;self.occupied[word]&=!(1<<(value%64));if self.occupied[word]==0{self.summary&=!(1<<word);}if self.len!=0{let word=(u8::BITS-1-self.summary.leading_zeros())as usize;self.maximum=(word*64+63-self.occupied[word].leading_zeros()as usize)as u8;}}Some(value as u8)}#[inline]fn replace(&mut self,value:u8)->Option<u8>{if self.len==0{self.push(value);return None;}let result=self.maximum;if value==result{return Some(result);}let old=result as usize;self.counts[old]-=1;if self.counts[old]==0{let word=old/64;self.occupied[word]&=!(1<<(old%64));if self.occupied[word]==0{self.summary&=!(1<<word);}}let new=value as usize;if self.counts[new]==0{self.occupied[new/64]|=1<<(new%64);self.summary|=1<<(new/64);}self.counts[new]+=1;if value>result||self.counts[old]==0{let word=(u8::BITS-1-self.summary.leading_zeros())as usize;self.maximum=(word*64+63-self.occupied[word].leading_zeros()as usize)as u8;}Some(result)}fn clear(&mut self){self.counts.fill(0);self.occupied.fill(0);self.summary=0;self.maximum=0;self.len=0;}}#[derive(Clone,Debug)]struct BucketQueue16{counts:Vec<u32>,occupied:Vec<u64>,summary:[u64;1<<4],top:u16,maximum:u16,len:usize}impl BucketQueue16{fn new()->Self{Self{counts:vec![0;1<<16],occupied:vec![0;1<<10],summary:[0;1<<4],top:0,maximum:0,len:0}}#[inline]fn push(&mut self,value:u16){assert!(self.len<u32::MAX as usize);let value=value as usize;if self.len==0||value>self.maximum as usize{self.maximum=value as u16;}if self.counts[value]==0{let word=value/64;self.occupied[word]|=1<<(value%64);self.summary[word/64]|=1<<(word%64);self.top|=1<<(word/64);}self.counts[value]+=1;self.len+=1;}fn from_values(values:impl IntoIterator<Item=u16>,len:usize)->Self{assert!(len<=u32::MAX as usize);let mut result=Self::new();result.len=len;for value in values{result.counts[value as usize]+=1;}for(value,&count)in result.counts.iter().enumerate(){if count!=0{result.occupied[value/64]|=1<<(value%64);}}for(word,&occupied)in result.occupied.iter().enumerate(){if occupied!=0{result.summary[word/64]|=1<<(word%64);}}for(word,&summary)in result.summary.iter().enumerate(){if summary!=0{result.top|=1<<word;}}if len!=0{let summary=(u16::BITS-1-result.top.leading_zeros())as usize;let word=summary*64+63-result.summary[summary].leading_zeros()as usize;result.maximum=(word*64+63-result.occupied[word].leading_zeros()as usize)as u16;}result}#[inline]fn pop(&mut self)->Option<u16>{if self.len==0{return None;}let value=self.maximum as usize;self.counts[value]-=1;self.len-=1;if self.counts[value]==0{let word=value/64;let summary=word/64;self.occupied[word]&=!(1<<(value%64));if self.occupied[word]==0{self.summary[summary]&=!(1<<(word%64));if self.summary[summary]==0{self.top&=!(1<<summary);}}if self.len!=0{let summary=(u16::BITS-1-self.top.leading_zeros())as usize;let word=summary*64+63-self.summary[summary].leading_zeros()as usize;self.maximum=(word*64+63-self.occupied[word].leading_zeros()as usize)as u16;}}Some(value as u16)}#[inline]fn replace(&mut self,value:u16)->Option<u16>{if self.len==0{self.push(value);return None;}let result=self.maximum;if value==result{return Some(result);}let old=result as usize;self.counts[old]-=1;if self.counts[old]==0{let word=old/64;let summary=word/64;self.occupied[word]&=!(1<<(old%64));if self.occupied[word]==0{self.summary[summary]&=!(1<<(word%64));if self.summary[summary]==0{self.top&=!(1<<summary);}}}let new=value as usize;if self.counts[new]==0{let word=new/64;self.occupied[word]|=1<<(new%64);self.summary[word/64]|=1<<(word%64);self.top|=1<<(word/64);}self.counts[new]+=1;if value>result||self.counts[old]==0{let summary=(u16::BITS-1-self.top.leading_zeros())as usize;let word=summary*64+63-self.summary[summary].leading_zeros()as usize;self.maximum=(word*64+63-self.occupied[word].leading_zeros()as usize)as u16;}Some(result)}fn clear(&mut self){self.counts.fill(0);self.occupied.fill(0);self.summary.fill(0);self.top=0;self.maximum=0;self.len=0;}}macro_rules!define_bucket_queue{($name:ident,$doc:literal,$value:ty,$repr:ty,$queue:ty,$sign:expr,$bulk_threshold:expr)=>{#[doc=$doc]#[derive(Clone,Debug)]pub struct$name{queue:$queue,}impl$name{pub fn new()->Self{Self{queue:<$queue>::new(),}}#[inline]pub fn len(&self)->usize{self.queue.len}#[inline]pub fn is_empty(&self)->bool{self.queue.len==0}#[inline]pub fn peek(&self)->Option<$value>{(self.queue.len!=0).then_some((self.queue.maximum^$sign)as$value)}#[doc=" # Panics"]#[doc=""]#[doc=" Panics if the queue already contains `u32::MAX` values."]#[inline]pub fn push(&mut self,value:$value){self.queue.push((value as$repr)^$sign);}#[inline]pub fn pop(&mut self)->Option<$value>{self.queue.pop().map(|value|((value^$sign)as$value))}#[doc=" Unconditionally replaces the greatest value, or inserts into an empty queue."]#[inline]pub fn replace(&mut self,value:$value)->Option<$value>{self.queue.replace((value as$repr)^$sign).map(|value|(value^$sign)as$value)}pub fn clear(&mut self){self.queue.clear();}}impl Default for$name{fn default()->Self{Self::new()}}impl From<Vec<$value>>for$name{fn from(values:Vec<$value>)->Self{if values.len()>=$bulk_threshold{let len=values.len();Self{queue:<$queue>::from_values(values.into_iter().map(|value|(value as$repr)^$sign),len,),}}else{let mut queue=Self::new();queue.extend(values);queue}}}impl Extend<$value>for$name{fn extend<I>(&mut self,iter:I)where I:IntoIterator<Item=$value>,{for value in iter{self.push(value);}}}impl FromIterator<$value>for$name{fn from_iter<I>(iter:I)->Self where I:IntoIterator<Item=$value>,{Self::from(Vec::from_iter(iter))}}};}define_bucket_queue!(BucketQueueU8,"A fixed 8-bit-universe max-priority queue. `BinaryHeap::peek_mut` can be faster for replacements in tiny queues.",u8,u8,BucketQueue8,0,1<<12);define_bucket_queue!(BucketQueueI8,"A fixed 8-bit-universe max-priority queue. `BinaryHeap::peek_mut` can be faster for replacements in tiny queues.",i8,u8,BucketQueue8,1<<7,1<<12);define_bucket_queue!(BucketQueueU16,"A fixed 16-bit-universe max-priority queue that allocates about 264 KiB when empty. `BinaryHeap` can be faster for small queues.",u16,u16,BucketQueue16,0,1<<16);define_bucket_queue!(BucketQueueI16,"A fixed 16-bit-universe max-priority queue that allocates about 264 KiB when empty. `BinaryHeap` can be faster for small queues.",i16,u16,BucketQueue16,1<<15,1<<16);}
// codesnip-guard: Comparator
#[cfg_attr(any(),rustfmt::skip)]pub use self::comparator::Comparator;#[cfg_attr(any(),rustfmt::skip)]pub mod comparator{use std::cmp::Ordering;pub trait Comparator<T>{fn compare(&mut self,a:&T,b:&T)->Ordering;}impl<T,F>Comparator<T>for F where F:FnMut(&T,&T)->Ordering{fn compare(&mut self,a:&T,b:&T)->Ordering{(self)(a,b)}}#[derive(Debug,Clone,Copy,Default,PartialEq,Eq,PartialOrd,Ord,Hash)]pub struct Less;impl<T>Comparator<T>for Less where T:Ord{fn compare(&mut self,a:&T,b:&T)->Ordering{a.cmp(b)}}#[derive(Debug,Clone,Copy,Default,PartialEq,Eq,PartialOrd,Ord,Hash)]pub struct Greater;impl<T>Comparator<T>for Greater where T:Ord{fn compare(&mut self,a:&T,b:&T)->Ordering{b.cmp(a)}}#[derive(Debug,Clone,Copy,Default,PartialEq,Eq,PartialOrd,Ord,Hash)]pub struct ByKey<F>(pub F);impl<T,F,K>Comparator<T>for ByKey<F>where F:FnMut(&T)->K,K:Ord{fn compare(&mut self,a:&T,b:&T)->Ordering{(self.0)(a).cmp(&(self.0)(b))}}}
// codesnip-guard: CompressedBinaryIndexedTree
#[cfg_attr(any(),rustfmt::skip)]pub use self::compressed_binary_indexed_tree::{CompressedBinaryIndexedTree,CompressedBinaryIndexedTree1d,CompressedBinaryIndexedTree2d,CompressedBinaryIndexedTree3d,CompressedBinaryIndexedTree4d};#[cfg_attr(any(),rustfmt::skip)]mod compressed_binary_indexed_tree{use super::{Monoid,SliceBisectExt};use std::{fmt::{self,Debug},marker::PhantomData,ops::{Bound,RangeBounds}};pub struct CompressedBinaryIndexedTree<M,X,Inner>where M:Monoid{compress:Vec<X>,bits:Vec<Inner>,_marker:PhantomData<fn()->M>}impl<M,X,Inner>Debug for CompressedBinaryIndexedTree<M,X,Inner>where M:Monoid,X:Debug,Inner:Debug{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{f.debug_struct("CompressedBinaryIndexedTree").field("compress",&self.compress).field("bits",&self.bits).finish()}}impl<M,X,Inner>Clone for CompressedBinaryIndexedTree<M,X,Inner>where M:Monoid,X:Clone,Inner:Clone{fn clone(&self)->Self{Self{compress:self.compress.clone(),bits:self.bits.clone(),_marker:self._marker}}}impl<M,X,Inner>Default for CompressedBinaryIndexedTree<M,X,Inner>where M:Monoid{fn default()->Self{Self{compress:Default::default(),bits:Default::default(),_marker:Default::default()}}}#[repr(transparent)]pub struct Tag<M>(M::T)where M:Monoid;impl<M>Debug for Tag<M>where M:Monoid<T:Debug>{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{self.0 .fmt(f)}}impl<M>Clone for Tag<M>where M:Monoid{fn clone(&self)->Self{Self(self.0 .clone())}}macro_rules!impl_compressed_binary_indexed_tree{(@tuple($($l:tt)*)($($r:tt)*)$T:ident)=>{($($l)*$T$($r)*,)};(@tuple($($l:tt)*)($($r:tt)*)$T:ident$($Rest:ident)+)=>{($($l)*$T$($r)*,impl_compressed_binary_indexed_tree!(@tuple($($l)*)($($r)*)$($Rest)+))};(@cst$M:ident)=>{Tag<$M>};(@cst$M:ident$T:ident$($Rest:ident)*)=>{CompressedBinaryIndexedTree<$M,$T,impl_compressed_binary_indexed_tree!(@cst$M$($Rest)*)>};(@from_iter$M:ident$points:ident$T:ident)=>{{let mut compress:Vec<_> =$points.into_iter().map(|t|t.0 .clone()).collect();compress.sort_unstable();compress.dedup();let n=compress.len();Self{compress,bits:vec![Tag(M::unit());n+1],_marker:PhantomData,}}};(@from_iter$M:ident$points:ident$T:ident$U:ident$($Rest:ident)*)=>{{let mut compress:Vec<_> =$points.clone().into_iter().map(|t|t.0 .clone()).collect();compress.sort_unstable();compress.dedup();let n=compress.len();let mut bits=vec![CompressedBinaryIndexedTree::default();n+1];let mut ps=vec![vec![];n+1];for(x,q)in$points{let i=compress.position_bisect(|c|x<=c);ps[i+1].push(q);}for i in 1..=n{bits[i]=CompressedBinaryIndexedTree::<_,_,impl_compressed_binary_indexed_tree!(@cst$M$($Rest)*)>::from_iter(ps[i].iter().cloned());let j=i+(i&(!i+1));if j<=n{let[s,ns]=ps.get_disjoint_mut([i,j]).unwrap();ns.append(s);}}Self{compress,bits,_marker:PhantomData,}}};(@acc$e:expr,$rng:ident$T:ident)=>{$e.0};(@acc$e:expr,$rng:ident$T:ident$($Rest:ident)+)=>{$e.accumulate(&$rng.1)};(@update$e:expr,$M:ident$key:ident$x:ident$T:ident)=>{$M::operate_assign(&mut$e.0,$x);};(@update$e:expr,$M:ident$key:ident$x:ident$T:ident$($Rest:ident)+)=>{$e.update(&$key.1,$x);};(@partition_method$T:ident,$Q:ident)=>{pub fn partition_point_acc<P>(&self,mut pred:P)->(Option<&$T>,M::T)where P:FnMut(&M::T)->bool,{let n=self.compress.len();let mut acc=M::unit();let mut pos=0;let mut k=n.next_power_of_two();if k>n{k>>=1;}while k>0{if k+pos<=n{let nacc=M::operate(&acc,&self.bits[k+pos].0);if pred(&nacc){pos+=k;acc=nacc;}}k>>=1;}(self.compress.get(pos),acc)}};(@partition_method$T:ident$($RestT:ident)+,$Q:ident$($RestQ:ident)+)=>{pub fn partition_point_acc<P,$($RestQ,)*>(&self,inner_ranges:&impl_compressed_binary_indexed_tree!(@tuple()()$($RestQ)*),mut pred:P,)->(Option<&$T>,M::T)where P:FnMut(&M::T)->bool,$($RestQ:RangeBounds<$RestT>,)*{let n=self.compress.len();let mut acc=M::unit();let mut pos=0;let mut k=n.next_power_of_two();if k>n{k>>=1;}while k>0{if k+pos<=n{let nacc=M::operate(&acc,&self.bits[k+pos].accumulate(inner_ranges),);if pred(&nacc){pos+=k;acc=nacc;}}k>>=1;}(self.compress.get(pos),acc)}};(@impl$C:ident$($T:ident)*,$($Q:ident)*)=>{impl<M,$($T,)*>impl_compressed_binary_indexed_tree!(@cst M$($T)*)where M:Monoid,$($T:Clone+Ord,)*{pub fn new(points:&[impl_compressed_binary_indexed_tree!(@tuple()()$($T)*)])->Self{Self::from_iter(points)}fn from_iter<'a,Iter>(points:Iter)->Self where$($T:'a,)*Iter:IntoIterator<Item=&'a impl_compressed_binary_indexed_tree!(@tuple()()$($T)*)>+Clone,{impl_compressed_binary_indexed_tree!(@from_iter M points$($T)*)}pub fn accumulate<$($Q,)*>(&self,range:&impl_compressed_binary_indexed_tree!(@tuple()()$($Q)*))->M::T where$($Q:RangeBounds<$T>,)*{match range.0 .start_bound(){Bound::Unbounded=>(),_=>panic!("expected `Bound::Unbounded`"),};let mut k=match range.0 .end_bound(){Bound::Included(index)=>self.compress.position_bisect(|x|x>&index),Bound::Excluded(index)=>self.compress.position_bisect(|x|x>=&index),Bound::Unbounded=>self.compress.len(),};let mut x=M::unit();while k>0{x=M::operate(&x,&impl_compressed_binary_indexed_tree!(@acc self.bits[k],range$($T)*));k-=k&(!k+1);}x}pub fn update(&mut self,key:&impl_compressed_binary_indexed_tree!(@tuple()()$($T)*),x:&M::T){let mut k=self.compress.binary_search(&key.0).expect("not exist key")+1;while k<self.bits.len(){impl_compressed_binary_indexed_tree!(@update self.bits[k],M key x$($T)*);k+=k&(!k+1);}}impl_compressed_binary_indexed_tree!(@partition_method$($T)*,$($Q)*);}pub type$C<M,$($T),*> =impl_compressed_binary_indexed_tree!(@cst M$($T)*);};(@inner[$C:ident][$($T:ident)*][$($Q:ident)*][])=>{impl_compressed_binary_indexed_tree!(@impl$C$($T)*,$($Q)*);};(@inner[$C:ident][$($T:ident)*][$($Q:ident)*][$D:ident$U:ident$R:ident$($Rest:ident)*])=>{impl_compressed_binary_indexed_tree!(@impl$C$($T)*,$($Q)*);impl_compressed_binary_indexed_tree!(@inner[$D][$($T)*$U][$($Q)*$R][$($Rest)*]);};($C:ident$T:ident$Q:ident$($Rest:ident)*$(;$($t:tt)*)?)=>{impl_compressed_binary_indexed_tree!(@inner[$C][$T][$Q][$($Rest)*]);};($($t:tt)*)=>{compile_error!($($t:tt)*)}}impl_compressed_binary_indexed_tree!(CompressedBinaryIndexedTree1d A QA CompressedBinaryIndexedTree2d B QB CompressedBinaryIndexedTree3d C QC CompressedBinaryIndexedTree4d D QD;CompressedBinaryIndexedTree5d E QE CompressedBinaryIndexedTree6d F QF CompressedBinaryIndexedTree7d G QG CompressedBinaryIndexedTree8d H QH CompressedBinaryIndexedTree9d I QI);}
// codesnip-guard: DaryHeap
#[cfg_attr(any(),rustfmt::skip)]pub use self::dary_heap::{DaryHeapI32,DaryHeapI64,DaryHeapI128,DaryHeapU32,DaryHeapU64,DaryHeapU128};#[cfg_attr(any(),rustfmt::skip)]mod dary_heap{#[cfg(target_arch="x86_64")]use super::simd;use super::{SimdBackend,simd_backend};#[repr(C,align(64))]#[derive(Clone,Debug)]struct HeapBlock<T,const D:usize>([T;D]);impl<T:Copy,const D:usize>HeapBlock<T,D>{#[inline(always)]fn filled(value:T)->Self{Self([value;D])}#[inline(always)]fn get(&self,index:usize)->T{unsafe{*self.0 .get_unchecked(index)}}#[inline(always)]fn set(&mut self,index:usize,value:T){unsafe{*self.0 .get_unchecked_mut(index)=value;}}}#[repr(C,align(64))]#[derive(Clone,Debug)]struct U128HeapBlock{low:[u64;4],high:[u64;4]}impl U128HeapBlock{#[inline(always)]fn filled(value:u128)->Self{Self{low:[value as u64;4],high:[(value>>64)as u64;4]}}#[inline(always)]fn get(&self,index:usize)->u128{unsafe{(*self.high.get_unchecked(index)as u128)<<64|*self.low.get_unchecked(index)as u128}}#[inline(always)]fn set(&mut self,index:usize,value:u128){unsafe{*self.low.get_unchecked_mut(index)=value as u64;*self.high.get_unchecked_mut(index)=(value>>64)as u64;}}}fn max_index<T:Ord,const D:usize>(values:&[T;D])->usize{let mut result=0;for index in 1..D{if values[index]>values[result]{result=index;}}result}fn max_index_u128(values:&U128HeapBlock)->usize{let mut result=0;for index in 1..4{if values.high[index]>values.high[result]||(values.high[index]==values.high[result]&&values.low[index]>values.low[result]){result=index;}}result}macro_rules!define_dary_heap{($name:ident,$doc:literal,$value:ty,$storage:ty,$branch:expr,$block:ty,encode=$encode:expr,decode=$decode:expr$(,$field:ident:$field_type:ty=$field_value:expr)*$(,)?)=>{#[doc=$doc]#[derive(Clone,Debug)]pub struct$name{root:$storage,blocks:Vec<$block>,len:usize,$(#[cfg(target_arch="x86_64")]$field:$field_type,)*}impl$name{pub fn new()->Self{Self::with_capacity(0)}pub fn with_capacity(capacity:usize)->Self{Self::empty(capacity$(,$field_value)*)}pub fn benchmark_empty(capacity:usize$(,$field:$field_type)*)->Self{Self::empty(capacity$(,$field)*)}pub fn benchmark_build(values:Vec<$value>$(,$field:$field_type)*)->Self{Self::build(values$(,$field)*)}#[inline]pub fn len(&self)->usize{self.len}#[inline]pub fn is_empty(&self)->bool{self.len==0}#[inline]pub fn peek(&self)->Option<$value>{(self.len!=0).then(||Self::decode(self.root))}pub fn push(&mut self,value:$value){let value=Self::encode(value);if self.len==0{self.root=value;self.len=1;return;}let mut hole=self.len;let block=(hole-1)/$branch;if block==self.blocks.len(){self.blocks.push(<$block>::filled(<$storage>::MIN));}self.len+=1;while hole!=0{let parent=(hole-1)/$branch;let parent_key=self.key(parent);if parent_key>=value{break;}self.set_key(hole,parent_key);hole=parent;}self.set_key(hole,value);}pub fn pop(&mut self)->Option<$value>{if self.len==0{return None;}let result=self.root;if self.len==1{self.root=<$storage>::MIN;self.len=0;return Some(Self::decode(result));}let last=self.len-1;let value=self.key(last);self.set_key(last,<$storage>::MIN);self.len=last;self.sift_down_after_pop(0,value);Some(Self::decode(result))}#[doc=" Unconditionally replaces the greatest value, or inserts into an empty heap."]pub fn replace(&mut self,value:$value)->Option<$value>{if self.len==0{self.push(value);return None;}let result=self.root;self.sift_down(0,Self::encode(value));Some(Self::decode(result))}pub fn clear(&mut self){self.root=<$storage>::MIN;self.blocks.clear();self.len=0;}pub fn into_sorted_vec(mut self)->Vec<$value>{let mut values=Vec::with_capacity(self.len);while let Some(value)=self.pop(){values.push(value);}values.reverse();values}fn empty(capacity:usize$(,$field:$field_type)*)->Self{$({let _=&$field;})*Self{root:<$storage>::MIN,blocks:Vec::with_capacity(capacity.saturating_sub(1).div_ceil($branch)),len:0,$(#[cfg(target_arch="x86_64")]$field,)*}}fn build(values:Vec<$value>$(,$field:$field_type)*)->Self{let len=values.len();let mut heap=Self::empty(len$(,$field)*);heap.len=len;if let Some((&root,values))=values.split_first(){heap.root=Self::encode(root);heap.blocks.resize(len.saturating_sub(1).div_ceil($branch),<$block>::filled(<$storage>::MIN),);for(index,&value)in values.iter().enumerate(){heap.blocks[index/$branch].set(index%$branch,Self::encode(value));}heap.heapify();}heap}#[inline(always)]fn encode(value:$value)->$storage{($encode)(value)}#[inline(always)]fn decode(value:$storage)->$value{($decode)(value)}#[inline(always)]fn key(&self,index:usize)->$storage{if index==0{self.root}else{unsafe{self.blocks.get_unchecked((index-1)/$branch)}.get((index-1)%$branch)}}#[inline(always)]fn set_key(&mut self,index:usize,value:$storage){if index==0{self.root=value;}else{unsafe{self.blocks.get_unchecked_mut((index-1)/$branch)}.set((index-1)%$branch,value);}}#[inline(always)]fn sift_down_by<F>(&mut self,mut hole:usize,value:$storage,mut max_index:F)where F:FnMut(&$block)->usize,{if self.len<=1{self.set_key(hole,value);return;}let last_parent=(self.len-2)/$branch;while hole<=last_parent{let block=unsafe{self.blocks.get_unchecked(hole)};let lane=max_index(block);let child_key=block.get(lane);if child_key<=value{break;}self.set_key(hole,child_key);hole=hole*$branch+lane+1;}self.set_key(hole,value);}fn heapify(&mut self){if self.len<=1{return;}for parent in(0..=(self.len-2)/$branch).rev(){let value=self.key(parent);self.sift_down(parent,value);}}}impl Default for$name{fn default()->Self{Self::new()}}impl From<Vec<$value>>for$name{fn from(values:Vec<$value>)->Self{Self::build(values$(,$field_value)*)}}impl Extend<$value>for$name{fn extend<I>(&mut self,iter:I)where I:IntoIterator<Item=$value>,{for value in iter{self.push(value);}}}impl FromIterator<$value>for$name{fn from_iter<I>(iter:I)->Self where I:IntoIterator<Item=$value>,{let values:Vec<_> =iter.into_iter().collect();Self::from(values)}}};}define_dary_heap!(DaryHeapU32,"A cache-line-oriented 16-ary max-heap for medium-to-large 32-bit heaps. `BinaryHeap` can be faster for small heaps and monotone replacements.",u32,u32,16,HeapBlock<u32,16>,encode=|value|value,decode=|value|value,backend:SimdBackend=simd_backend(),);define_dary_heap!(DaryHeapI32,"A cache-line-oriented 16-ary max-heap for medium-to-large 32-bit heaps. `BinaryHeap` can be faster for small heaps and monotone replacements.",i32,u32,16,HeapBlock<u32,16>,encode=|value:i32|value as u32^(1<<31),decode=|value:u32|(value^(1<<31))as i32,backend:SimdBackend=simd_backend(),);define_dary_heap!(DaryHeapU64,"A cache-line-oriented 8-ary max-heap for large 64-bit heaps. `BinaryHeap` can be faster for small heaps and monotone replacements.",u64,u64,8,HeapBlock<u64,8>,encode=|value|value,decode=|value|value,backend:SimdBackend=simd_backend(),);define_dary_heap!(DaryHeapI64,"A cache-line-oriented 8-ary max-heap for large 64-bit heaps. `BinaryHeap` can be faster for small heaps and monotone replacements.",i64,u64,8,HeapBlock<u64,8>,encode=|value:i64|value as u64^(1<<63),decode=|value:u64|(value^(1<<63))as i64,backend:SimdBackend=simd_backend(),);define_dary_heap!(DaryHeapU128,"A cache-line-oriented 4-ary max-heap for large full-width 128-bit heaps. `BinaryHeap` can be faster for small heaps, monotone replacements, and heavily repeated keys.",u128,u128,4,U128HeapBlock,encode=|value|value,decode=|value|value,backend:SimdBackend=simd_backend(),);define_dary_heap!(DaryHeapI128,"A cache-line-oriented 4-ary max-heap for large full-width 128-bit heaps. `BinaryHeap` can be faster for small heaps, monotone replacements, and heavily repeated keys.",i128,u128,4,U128HeapBlock,encode=|value:i128|value as u128^(1<<127),decode=|value:u128|(value^(1<<127))as i128,backend:SimdBackend=simd_backend(),);macro_rules!impl_simd_heap{($name:ident,$value:ty,$branch:expr,$max_avx2:ident,$max_avx512:ident)=>{impl$name{#[inline(always)]fn sift_down_scalar(&mut self,hole:usize,value:$value){self.sift_down_by(hole,value,|block|max_index(&block.0))}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]unsafe fn sift_down_avx2(&mut self,hole:usize,value:$value){self.sift_down_by(hole,value,|block|unsafe{simd::$max_avx2(&block.0)})}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f")]unsafe fn sift_down_avx512(&mut self,hole:usize,value:$value){self.sift_down_by(hole,value,|block|unsafe{simd::$max_avx512(&block.0)})}#[inline]fn sift_down(&mut self,hole:usize,value:$value){#[cfg(target_arch="x86_64")]match self.backend{SimdBackend::Scalar=>self.sift_down_scalar(hole,value),SimdBackend::Avx2=>unsafe{self.sift_down_avx2(hole,value)},SimdBackend::Avx512=>unsafe{self.sift_down_avx512(hole,value)},}#[cfg(not(target_arch="x86_64"))]self.sift_down_scalar(hole,value);}#[inline(always)]fn sift_down_after_pop(&mut self,hole:usize,value:$value){self.sift_down(hole,value);}}};}impl_simd_heap!(DaryHeapU32,u32,16,max_index_u32x16_avx2,max_index_u32x16_avx512);impl_simd_heap!(DaryHeapI32,u32,16,max_index_u32x16_avx2,max_index_u32x16_avx512);impl_simd_heap!(DaryHeapU64,u64,8,max_index_u64x8_avx2,max_index_u64x8_avx512);impl_simd_heap!(DaryHeapI64,u64,8,max_index_u64x8_avx2,max_index_u64x8_avx512);macro_rules!impl_u128_heap{($name:ident)=>{impl$name{#[inline(always)]fn sift_down_scalar(&mut self,hole:usize,value:u128){self.sift_down_by(hole,value,max_index_u128)}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]unsafe fn sift_down_avx2(&mut self,hole:usize,value:u128){self.sift_down_by(hole,value,|block|unsafe{simd::max_index_u128x4_avx2(&block.low,&block.high)})}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2,avx512f,avx512vl")]unsafe fn sift_down_avx512(&mut self,hole:usize,value:u128){self.sift_down_by(hole,value,|block|unsafe{simd::max_index_u128x4_avx512(&block.low,&block.high)})}#[inline]fn sift_down(&mut self,hole:usize,value:u128){#[cfg(target_arch="x86_64")]match self.backend{SimdBackend::Scalar=>self.sift_down_scalar(hole,value),SimdBackend::Avx2=>unsafe{self.sift_down_avx2(hole,value)},SimdBackend::Avx512=>unsafe{self.sift_down_avx512(hole,value)},}#[cfg(not(target_arch="x86_64"))]self.sift_down_scalar(hole,value);}#[inline]fn sift_down_after_pop(&mut self,hole:usize,value:u128){#[cfg(target_arch="x86_64")]match self.backend{SimdBackend::Scalar=>self.sift_down_scalar(hole,value),SimdBackend::Avx2 if self.len<1<<18=>self.sift_down_scalar(hole,value),SimdBackend::Avx2=>unsafe{self.sift_down_avx2(hole,value)},SimdBackend::Avx512 if self.len<1<<15=>self.sift_down_scalar(hole,value),SimdBackend::Avx512=>unsafe{self.sift_down_avx512(hole,value)},}#[cfg(not(target_arch="x86_64"))]self.sift_down_scalar(hole,value);}}};}impl_u128_heap!(DaryHeapU128);impl_u128_heap!(DaryHeapI128);}
// codesnip-guard: LastOperation
#[cfg_attr(any(),rustfmt::skip)]pub use self::last_operation_impl::LastOperation;#[cfg_attr(any(),rustfmt::skip)]mod last_operation_impl{use super::*;use std::marker::PhantomData;#[doc=" retain the last element"]pub struct LastOperation<T>where T:Clone{_marker:PhantomData<fn()->T>}impl<T>Magma for LastOperation<T>where T:Clone{type T=Option<T>;#[inline]fn operate(x:&Self::T,y:&Self::T)->Self::T{y.as_ref().or(x.as_ref()).cloned()}}impl<T>Unital for LastOperation<T>where T:Clone{#[inline]fn unit()->Self::T{None}}impl<T>Associative for LastOperation<T>where T:Clone{}impl<T>Idempotent for LastOperation<T>where T:Clone{}}
// codesnip-guard: LinearOperation
#[cfg_attr(any(),rustfmt::skip)]pub use self::linear_operation_impl::LinearOperation;#[cfg_attr(any(),rustfmt::skip)]mod linear_operation_impl{use super::*;use std::{marker::PhantomData,ops::{Add,Div,Mul,Neg,Sub}};#[doc=" $(a, b) \\circ (c, d) = \\lambda x. c \\times (a \\times x + b) + d$"]pub struct LinearOperation<T>where T:Clone+Zero+Add<Output=T>+One+Mul<Output=T>{_marker:PhantomData<fn()->T>}impl<T>LinearOperation<T>where T:Clone+Zero+Add<Output=T>+One+Mul<Output=T>{pub fn apply(f:&(T,T),x:&T)->T{f.0 .clone()*x.clone()+f.1 .clone()}}impl<T>Magma for LinearOperation<T>where T:Clone+Zero+One+Add<Output=T>+Mul<Output=T>{type T=(T,T);#[inline]fn operate(x:&Self::T,y:&Self::T)->Self::T{(y.0 .clone()*x.0 .clone(),y.0 .clone()*x.1 .clone()+y.1 .clone())}}impl<T>Unital for LinearOperation<T>where T:Clone+Zero+One+Add<Output=T>+Mul<Output=T>{#[inline]fn unit()->Self::T{(One::one(),Zero::zero())}}impl<T>Associative for LinearOperation<T>where T:Clone+Zero+One+Add<Output=T>+Mul<Output=T>{}impl<T>Invertible for LinearOperation<T>where T:Clone+Zero+One+Add<Output=T>+Sub<Output=T>+Neg<Output=T>+Mul<Output=T>+Div<Output=T>{fn inverse(x:&Self::T)->Self::T{let y=<T as One>::one().div(x.0 .clone());(y.clone(),-y.mul(x.1 .clone()))}}}
// codesnip-guard: MaxOperation
#[cfg_attr(any(),rustfmt::skip)]pub use self::max_operation_impl::MaxOperation;#[cfg_attr(any(),rustfmt::skip)]mod max_operation_impl{use super::*;use std::marker::PhantomData;#[doc=" binary operation to select larger element"]pub struct MaxOperation<T>where T:Clone+Ord+Bounded{_marker:PhantomData<fn()->T>}impl<T>Magma for MaxOperation<T>where T:Clone+Ord+Bounded{type T=T;#[inline]fn operate(x:&Self::T,y:&Self::T)->Self::T{x.max(y).clone()}}impl<T>Unital for MaxOperation<T>where T:Clone+Ord+Bounded{#[inline]fn unit()->Self::T{<T as Bounded>::minimum()}}impl<T>Associative for MaxOperation<T>where T:Clone+Ord+Bounded{}impl<T>Commutative for MaxOperation<T>where T:Clone+Ord+Bounded{}impl<T>Idempotent for MaxOperation<T>where T:Clone+Ord+Bounded{}}
// codesnip-guard: MinOperation
#[cfg_attr(any(),rustfmt::skip)]pub use self::min_operation_impl::MinOperation;#[cfg_attr(any(),rustfmt::skip)]mod min_operation_impl{use super::*;use std::marker::PhantomData;#[doc=" binary operation to select smaller element"]pub struct MinOperation<T>where T:Clone+Ord+Bounded{_marker:PhantomData<fn()->T>}impl<T>Magma for MinOperation<T>where T:Clone+Ord+Bounded{type T=T;#[inline]fn operate(x:&Self::T,y:&Self::T)->Self::T{x.min(y).clone()}}impl<T>Unital for MinOperation<T>where T:Clone+Ord+Bounded{#[inline]fn unit()->Self::T{<T as Bounded>::maximum()}}impl<T>Associative for MinOperation<T>where T:Clone+Ord+Bounded{}impl<T>Commutative for MinOperation<T>where T:Clone+Ord+Bounded{}impl<T>Idempotent for MinOperation<T>where T:Clone+Ord+Bounded{}}
// codesnip-guard: MonoidAct
#[cfg_attr(any(),rustfmt::skip)]pub use self::action::*;#[cfg_attr(any(),rustfmt::skip)]mod action{use super::*;use std::{marker::PhantomData,ops::{Add,Mul}};pub trait MonoidAct{type Key;type Act:Clone;type ActMonoid:Monoid<T=Self::Act>;fn act(x:&Self::Key,a:&Self::Act)->Self::Key;fn act_assign(x:&mut Self::Key,a:&Self::Act){*x=Self::act(x,a);}fn unit()->Self::Act{<Self::ActMonoid as Unital>::unit()}fn operate(x:&Self::Act,y:&Self::Act)->Self::Act{<Self::ActMonoid as Magma>::operate(x,y)}fn operate_assign(x:&mut Self::Act,y:&Self::Act){*x=<Self::ActMonoid as Magma>::operate(x,y);}}pub struct EmptyAct<T>{_marker:PhantomData<fn()->T>}impl<T>MonoidAct for EmptyAct<T>where T:Clone{type Key=T;type Act=();type ActMonoid=();fn act(x:&Self::Key,_a:&Self::Act)->Self::Key{x.clone()}}pub struct FlattenAct<M>{_marker:PhantomData<fn()->M>}impl<M>MonoidAct for FlattenAct<M>where M:Monoid{type Key=M::T;type Act=M::T;type ActMonoid=M;fn act(x:&Self::Key,a:&Self::Act)->Self::Key{M::operate(x,a)}}pub struct LinearAct<T>{_marker:PhantomData<fn()->T>}impl<T>MonoidAct for LinearAct<T>where T:Clone+Zero+One+Add<Output=T>+Mul<Output=T>{type Key=T;type Act=(T,T);type ActMonoid=LinearOperation<T>;fn act(x:&Self::Key,(a,b):&Self::Act)->Self::Key{a.clone()*x.clone()+b.clone()}}pub struct UpdateAct<T>{_marker:PhantomData<fn()->T>}impl<T>MonoidAct for UpdateAct<T>where T:Clone{type Key=T;type Act=Option<T>;type ActMonoid=LastOperation<T>;fn act(x:&Self::Key,a:&Self::Act)->Self::Key{a.as_ref().unwrap_or(x).clone()}}}
// codesnip-guard: PairingHeap
#[cfg_attr(any(),rustfmt::skip)]pub use self::pairing_heap::PairingHeap;#[cfg_attr(any(),rustfmt::skip)]mod pairing_heap{use super::{Comparator,EmptyAct,MonoidAct,Unital,comparator::Less};use std::{cmp::Ordering,fmt::{self,Debug,Formatter},iter::FusedIterator,mem::{replace,swap},ops::{Deref,DerefMut}};#[derive(Clone)]struct Node<T,A>where A:MonoidAct<Key=T,Act:PartialEq>{value:T,first_child:Option<Box<Node<T,A>>>,next_sibling:Option<Box<Node<T,A>>>,lazy:A::Act}impl<T,A>Node<T,A>where A:MonoidAct<Key=T,Act:PartialEq>{fn new(value:T)->Self{Self{value,first_child:None,next_sibling:None,lazy:A::unit()}}fn apply(&mut self,act:&A::Act){A::act_assign(&mut self.value,act);A::operate_assign(&mut self.lazy,act);}fn propagate(&mut self){if!<A::ActMonoid as Unital>::is_unit(&self.lazy){let act=replace(&mut self.lazy,A::unit());if let Some(node)=self.first_child.as_mut(){node.apply(&act);}if let Some(node)=self.next_sibling.as_mut(){node.apply(&act);}}}}impl<T,A>Debug for Node<T,A>where T:Debug,A:MonoidAct<Key=T,Act:PartialEq+Debug>{fn fmt(&self,f:&mut Formatter<'_>)->fmt::Result{f.debug_struct("Node").field("value",&self.value).field("first_child",&self.first_child).field("next_sibling",&self.next_sibling).field("lazy",&self.lazy).finish()}}#[derive(Clone)]pub struct PairingHeap<T,C=Less,A=EmptyAct<T>>where A:MonoidAct<Key=T,Act:PartialEq>{root:Option<Box<Node<T,A>>>,len:usize,cmp:C}impl<T,C,A>PairingHeap<T,C,A>where C:Comparator<T>,A:MonoidAct<Key=T,Act:PartialEq>{pub fn with_comparator(cmp:C)->Self{Self{root:None,len:0,cmp}}pub fn len(&self)->usize{self.len}pub fn is_empty(&self)->bool{self.len==0}pub fn peek(&self)->Option<&T>{self.root.as_ref().map(|node|&node.value)}pub fn push(&mut self,value:T){let node=Box::new(Node::new(value));let root=self.root.take();self.root=self.merge_option(root,Some(node));self.len+=1;}pub fn append(&mut self,other:&mut Self){if other.is_empty(){return;}let left=self.root.take();self.root=self.merge_option(left,other.root.take());self.len+=other.len;other.len=0;}pub fn pop(&mut self)->Option<T>{self.root.take().map(|mut root|{self.len-=1;root.propagate();let children=root.first_child.take();self.root=self.merge_pairs(children);root.value})}pub fn peek_mut(&mut self)->Option<PeekMut<'_,T,C,A>>{let mut root=self.root.take()?;root.propagate();let children=root.first_child.take();debug_assert!(root.next_sibling.is_none());root.next_sibling=None;self.root=self.merge_pairs(children);Some(PeekMut{heap:self,node:Some(root)})}pub fn clear(&mut self){self.root=None;self.len=0;}pub fn apply_all(&mut self,act:A::Act){if let Some(root)=self.root.as_mut(){root.apply(&act);}}pub fn into_sorted_vec(mut self)->Vec<T>{let mut result=Vec::with_capacity(self.len);while let Some(value)=self.pop(){result.push(value);}result}fn merge_option(&mut self,a:Option<Box<Node<T,A>>>,b:Option<Box<Node<T,A>>>)->Option<Box<Node<T,A>>>{match(a,b){(None,None)=>None,(Some(node),None)|(None,Some(node))=>Some(node),(Some(mut a),Some(mut b))=>{a.propagate();b.propagate();if self.cmp.compare(&a.value,&b.value)==Ordering::Greater{swap(&mut a,&mut b);}b.next_sibling=a.first_child.take();a.first_child=Some(b);Some(a)}}}fn merge_pairs(&mut self,mut head:Option<Box<Node<T,A>>>)->Option<Box<Node<T,A>>>{let mut pairs:Vec<Box<Node<T,A>>>=Vec::new();while let Some(mut first)=head{first.propagate();let next=first.next_sibling.take();if let Some(mut second)=next{second.propagate();head=second.next_sibling.take();pairs.push(self.merge_option(Some(first),Some(second)).unwrap());}else{pairs.push(first);break;}}let mut result=None;while let Some(node)=pairs.pop(){result=self.merge_option(Some(node),result);}result}}impl<T,C,A>Default for PairingHeap<T,C,A>where C:Comparator<T>+Default,A:MonoidAct<Key=T,Act:PartialEq>{fn default()->Self{Self::with_comparator(C::default())}}impl<T,A>PairingHeap<T,Less,A>where T:Ord,A:MonoidAct<Key=T,Act:PartialEq>{pub fn new()->Self{Self::default()}}impl<T,C,A>Debug for PairingHeap<T,C,A>where T:Debug,C:Debug+Comparator<T>,A:MonoidAct<Key=T,Act:PartialEq+Debug>{fn fmt(&self,f:&mut Formatter<'_>)->fmt::Result{f.debug_struct("PairingHeap").field("len",&self.len).field("root",&self.root).field("cmp",&self.cmp).finish()}}impl<T,C,A>Extend<T>for PairingHeap<T,C,A>where C:Comparator<T>,A:MonoidAct<Key=T,Act:PartialEq>{fn extend<I:IntoIterator<Item=T>>(&mut self,iter:I){for value in iter{self.push(value);}}}impl<T,C,A>FromIterator<T>for PairingHeap<T,C,A>where C:Comparator<T>+Default,A:MonoidAct<Key=T,Act:PartialEq>{fn from_iter<I:IntoIterator<Item=T>>(iter:I)->Self{let mut heap=Self::default();heap.extend(iter);heap}}pub struct PeekMut<'a,T,C=Less,A=EmptyAct<T>>where C:Comparator<T>,A:MonoidAct<Key=T,Act:PartialEq>{heap:&'a mut PairingHeap<T,C,A>,node:Option<Box<Node<T,A>>>}impl<'a,T,C,A>PeekMut<'a,T,C,A>where C:Comparator<T>,A:MonoidAct<Key=T,Act:PartialEq>{pub fn pop(mut this:Self)->T{this.heap.len-=1;let node=this.node.take().expect("PeekMut already consumed");let Node{value,..}=*node;value}}impl<'a,T,C,A>Deref for PeekMut<'a,T,C,A>where C:Comparator<T>,A:MonoidAct<Key=T,Act:PartialEq>{type Target=T;fn deref(&self)->&Self::Target{&self.node.as_ref().expect("PeekMut already consumed").value}}impl<'a,T,C,A>DerefMut for PeekMut<'a,T,C,A>where C:Comparator<T>,A:MonoidAct<Key=T,Act:PartialEq>{fn deref_mut(&mut self)->&mut Self::Target{&mut self.node.as_mut().expect("PeekMut already consumed").value}}impl<'a,T,C,A>Drop for PeekMut<'a,T,C,A>where C:Comparator<T>,A:MonoidAct<Key=T,Act:PartialEq>{fn drop(&mut self){if let Some(mut node)=self.node.take(){debug_assert!(node.next_sibling.is_none());let root=self.heap.root.take();node.first_child=None;self.heap.root=self.heap.merge_option(root,Some(node));}}}pub struct IntoIter<T,C=Less,A=EmptyAct<T>>where C:Comparator<T>,A:MonoidAct<Key=T,Act:PartialEq>{heap:PairingHeap<T,C,A>}impl<T,C,A>IntoIter<T,C,A>where C:Comparator<T>,A:MonoidAct<Key=T,Act:PartialEq>{fn new(heap:PairingHeap<T,C,A>)->Self{Self{heap}}}impl<T,C,A>Iterator for IntoIter<T,C,A>where C:Comparator<T>,A:MonoidAct<Key=T,Act:PartialEq>{type Item=T;fn next(&mut self)->Option<Self::Item>{self.heap.pop()}fn size_hint(&self)->(usize,Option<usize>){let len=self.heap.len();(len,Some(len))}}impl<T,C,A>ExactSizeIterator for IntoIter<T,C,A>where C:Comparator<T>,A:MonoidAct<Key=T,Act:PartialEq>{fn len(&self)->usize{self.heap.len()}}impl<T,C,A>FusedIterator for IntoIter<T,C,A>where C:Comparator<T>,A:MonoidAct<Key=T,Act:PartialEq>{}impl<T,C,A>IntoIterator for PairingHeap<T,C,A>where C:Comparator<T>,A:MonoidAct<Key=T,Act:PartialEq>{type Item=T;type IntoIter=IntoIter<T,C,A>;fn into_iter(self)->Self::IntoIter{IntoIter::new(self)}}}
// codesnip-guard: RadixHeapU64
#[cfg_attr(any(),rustfmt::skip)]pub use self::radix_heap::RadixHeapU64;#[cfg_attr(any(),rustfmt::skip)]mod radix_heap{#[doc=" A min-priority queue whose removed `u64` keys are monotonically nondecreasing."]#[doc=""]#[doc=" Values with equal keys have no specified removal order."]#[derive(Clone,Debug)]pub struct RadixHeapU64<T>{buckets:[Vec<(u64,T)>;65],last:u64,len:usize}impl<T>RadixHeapU64<T>{pub fn new()->Self{Self{buckets:std::array::from_fn(|_|Vec::new()),last:0,len:0}}pub fn len(&self)->usize{self.len}pub fn is_empty(&self)->bool{self.len==0}#[doc=" Inserts a value whose key is not less than the key most recently removed."]#[doc=""]#[doc=" # Panics"]#[doc=""]#[doc=" Panics if `key` is less than the key most recently removed."]pub fn push(&mut self,key:u64,value:T){assert!(key>=self.last,"key is less than the last removed key");self.buckets[Self::bucket_index(key,self.last)].push((key,value));self.len+=1;}pub fn pop(&mut self)->Option<(u64,T)>{if self.len==0{return None;}if self.buckets[0].is_empty(){let index=(1..self.buckets.len()).find(|&index|!self.buckets[index].is_empty()).unwrap();self.last=self.buckets[index].iter().map(|&(key,_)|key).min().unwrap();let values=std::mem::take(&mut self.buckets[index]);for(key,value)in values{self.buckets[Self::bucket_index(key,self.last)].push((key,value));}}self.len-=1;self.buckets[0].pop()}pub fn clear(&mut self){for bucket in&mut self.buckets{bucket.clear();}self.last=0;self.len=0;}#[inline]fn bucket_index(key:u64,last:u64)->usize{(u64::BITS-(key^last).leading_zeros())as usize}}impl<T>Default for RadixHeapU64<T>{fn default()->Self{Self::new()}}}
// codesnip-guard: RangeMinimumQuery
#[cfg_attr(any(),rustfmt::skip)]pub use self::range_minimum_query::RangeMinimumQuery;#[cfg_attr(any(),rustfmt::skip)]mod range_minimum_query{const BLOCK_SIZE:usize=64;#[derive(Clone,Debug)]pub struct RangeMinimumQuery<T>{data:Vec<T>,suffix:Vec<T>,prefix:Vec<T>,table:Vec<T>,blocks:usize}impl<T>RangeMinimumQuery<T>where T:Ord+Copy{pub fn new(data:Vec<T>)->Self{let n=data.len();let blocks=n.div_ceil(BLOCK_SIZE);if blocks==0{return Self{data,suffix:vec![],prefix:vec![],table:vec![],blocks};}let levels=usize::BITS as usize-blocks.leading_zeros()as usize;let mut table=vec![data[0];levels*blocks];let mut prefix=Vec::with_capacity(n);let mut minimum=data[0];for(i,&value)in data.iter().enumerate(){minimum=if i%BLOCK_SIZE==0{value}else{minimum.min(value)};prefix.push(minimum);if i%BLOCK_SIZE==BLOCK_SIZE-1||i+1==n{table[i/BLOCK_SIZE]=minimum;}}let mut suffix=data.clone();for block in suffix.chunks_mut(BLOCK_SIZE){minimum=*block.last().unwrap();for value in block.iter_mut().rev(){minimum=minimum.min(*value);*value=minimum;}}for level in 1..levels{let current=level*blocks;let previous=current-blocks;let half=1<<(level-1);for i in 0..blocks-(1<<level)+1{table[current+i]=if table[previous+i]<table[previous+i+half]{table[previous+i]}else{table[previous+i+half]};}}Self{data,suffix,prefix,table,blocks}}#[inline]pub fn fold(&self,l:usize,r:usize)->T{let r=r-1;let left_block=l/BLOCK_SIZE;let right_block=r/BLOCK_SIZE;if left_block+1<right_block{let middle_blocks=right_block-left_block-1;let level=middle_blocks.ilog2()as usize;let offset=level*self.blocks;self.suffix[l].min(self.prefix[r]).min(self.table[offset+left_block+1]).min(self.table[offset+right_block-(1<<level)])}else if left_block<right_block{self.suffix[l].min(self.prefix[r])}else{*self.data[l..=r].iter().min().unwrap()}}}}
// codesnip-guard: SegmentTree
#[cfg_attr(any(),rustfmt::skip)]pub use self::segment_tree::SegmentTree;#[cfg_attr(any(),rustfmt::skip)]mod segment_tree{use super::{AbelianMonoid,Monoid,RangeBoundsExt};use std::{fmt::{self,Debug,Formatter},ops::RangeBounds};pub struct SegmentTree<M>where M:Monoid{n:usize,seg:Vec<M::T>}impl<M>Clone for SegmentTree<M>where M:Monoid{fn clone(&self)->Self{Self{n:self.n,seg:self.seg.clone()}}}impl<M>Debug for SegmentTree<M>where M:Monoid<T:Debug>{fn fmt(&self,f:&mut Formatter<'_>)->fmt::Result{f.debug_struct("SegmentTree").field("n",&self.n).field("seg",&self.seg).finish()}}impl<M>SegmentTree<M>where M:Monoid{pub fn new(n:usize)->Self{let seg=vec![M::unit();2*n];Self{n,seg}}pub fn from_vec(v:Vec<M::T>)->Self{let n=v.len();let mut seg=vec![M::unit();2*n];for(i,x)in v.into_iter().enumerate(){seg[n+i]=x;}for i in(1..n).rev(){seg[i]=M::operate(&seg[2*i],&seg[2*i+1]);}Self{n,seg}}pub fn set(&mut self,k:usize,x:M::T){assert!(k<self.n);let mut k=k+self.n;self.seg[k]=x;k/=2;while k>0{self.seg[k]=M::operate(&self.seg[2*k],&self.seg[2*k+1]);k/=2;}}pub fn clear(&mut self,k:usize){self.set(k,M::unit());}pub fn update(&mut self,k:usize,x:M::T){assert!(k<self.n);let mut k=k+self.n;self.seg[k]=M::operate(&self.seg[k],&x);k/=2;while k>0{self.seg[k]=M::operate(&self.seg[2*k],&self.seg[2*k+1]);k/=2;}}pub fn get(&self,k:usize)->M::T{assert!(k<self.n);self.seg[k+self.n].clone()}pub fn fold<R>(&self,range:R)->M::T where R:RangeBounds<usize>{let range=range.to_range_bounded(0,self.n).expect("invalid range");let mut l=range.start+self.n;let mut r=range.end+self.n;let mut vl=M::unit();let mut vr=M::unit();while l<r{if l&1!=0{vl=M::operate(&vl,&self.seg[l]);l+=1;}if r&1!=0{r-=1;vr=M::operate(&self.seg[r],&vr);}l/=2;r/=2;}M::operate(&vl,&vr)}fn partition_point_perfect<P>(&self,mut pos:usize,mut acc:M::T,mut pred:P)->(usize,M::T)where P:FnMut(&M::T)->bool{while pos<self.n{pos<<=1;let nacc=M::operate(&acc,&self.seg[pos]);if pred(&nacc){acc=nacc;pos+=1;}}(pos-self.n,acc)}fn rpartition_point_perfect<P>(&self,mut pos:usize,mut acc:M::T,mut pred:P)->(usize,M::T)where P:FnMut(&M::T)->bool{while pos<self.n{pos=pos*2+1;let nacc=M::operate(&self.seg[pos],&acc);if pred(&nacc){acc=nacc;pos-=1;}}(pos-self.n,acc)}pub fn partition_point_acc<P>(&self,left:usize,mut pred:P)->usize where P:FnMut(&M::T)->bool{let mut l=left+self.n;let r=2*self.n;let mut k=0usize;let mut acc=M::unit();while l<r>>k{if l&1!=0{let nacc=M::operate(&acc,&self.seg[l]);if!pred(&nacc){return self.partition_point_perfect(l,acc,pred).0;}acc=nacc;l+=1;}l>>=1;k+=1;}for k in(0..k).rev(){let r=r>>k;if r&1!=0{let nacc=M::operate(&acc,&self.seg[r-1]);if!pred(&nacc){return self.partition_point_perfect(r-1,acc,pred).0;}acc=nacc;}}self.n}pub fn rpartition_point_acc<P>(&self,right:usize,mut pred:P)->usize where P:FnMut(&M::T)->bool{let mut l=self.n;let mut r=right+self.n;let mut c=0usize;let mut k=0usize;let mut acc=M::unit();while l>>k<r{c<<=1;if l&(1<<k)!=0{l+=1<<k;c+=1;}if r&1!=0{r-=1;let nacc=M::operate(&self.seg[r],&acc);if!pred(&nacc){return self.rpartition_point_perfect(r,acc,pred).0+1;}acc=nacc;}r>>=1;k+=1;}for k in(0..k).rev(){if c&1!=0{l-=1<<k;let l=l>>k;let nacc=M::operate(&self.seg[l],&acc);if!pred(&nacc){return self.rpartition_point_perfect(l,acc,pred).0+1;}acc=nacc;}c>>=1;}0}pub fn as_slice(&self)->&[M::T]{&self.seg[self.n..]}}impl<M>SegmentTree<M>where M:AbelianMonoid{pub fn fold_all(&self)->M::T{self.seg[1].clone()}}}
// codesnip-guard: StaticSearch
#[cfg_attr(any(),rustfmt::skip)]pub use self::static_search::{SimdKey,StaticSearch};#[cfg_attr(any(),rustfmt::skip)]mod static_search{use super::SimdBackend;#[cfg(target_arch="x86_64")]use super::{avx512_enabled,simd};use std::{marker::PhantomData,ops::Range};fn static_search_backend(bits:u32)->SimdBackend{#[cfg(target_arch="x86_64")]{if avx512_enabled()&&is_x86_feature_detected!("avx512f")&&(bits!=16||is_x86_feature_detected!("avx512bw")){return SimdBackend::Avx512;}if is_x86_feature_detected!("avx2"){return SimdBackend::Avx2;}}let _=bits;SimdBackend::Scalar}#[doc=" Maps a key to an unsigned integer with exactly the same ordering."]#[doc=""]#[doc=" `BITS` must be 8, 16, 32, 64, or 128. `encode` must be deterministic, fit in"]#[doc=" `BITS`, and satisfy `a.cmp(&b) == a.encode().cmp(&b.encode())`."]pub trait SimdKey:Copy+Ord{const BITS:u32;fn encode(self)->u128;}macro_rules!impl_unsigned_simd_key{($($value:ty),*$(,)?)=>{$(impl SimdKey for$value{const BITS:u32=<$value>::BITS;#[inline(always)]fn encode(self)->u128{self as u128}})*};}macro_rules!impl_signed_simd_key{($(($signed:ty,$unsigned:ty)),*$(,)?)=>{$(impl SimdKey for$signed{const BITS:u32=<$signed>::BITS;#[inline(always)]fn encode(self)->u128{((self as$unsigned)^(1 as$unsigned<<(<$signed>::BITS-1)))as u128}})*};}impl_unsigned_simd_key!(u8,u16,u32,u64,u128,usize);impl_signed_simd_key!((i8,u8),(i16,u16),(i32,u32),(i64,u64),(i128,u128),(isize,usize),);#[derive(Clone,Debug)]enum DirectStaticSearch{U16(Vec<u16>),U32(Vec<u32>)}impl DirectStaticSearch{fn build<K:SimdKey>(values:&[K],bits:u32)->Self{assert!(values.len()<=u32::MAX as usize);if values.len()<=u16::MAX as usize{Self::U16(build_direct_positions(values,bits,|position|{position as u16}))}else{Self::U32(build_direct_positions(values,bits,|position|{position as u32}))}}#[inline(always)]fn position_bisect(&self,value:u128)->usize{let value=usize::try_from(value).expect("SimdKey::encode exceeds usize");match self{Self::U16(positions)=>positions[value]as usize,Self::U32(positions)=>positions[value]as usize,}}#[inline(always)]fn rposition_bisect(&self,value:u128)->usize{let value=usize::try_from(value).ok().and_then(|value|value.checked_add(1)).expect("SimdKey::encode exceeds usize");match self{Self::U16(positions)=>positions[value]as usize,Self::U32(positions)=>positions[value]as usize,}}#[inline(always)]fn contains(&self,value:u128)->bool{let value=usize::try_from(value).ok().and_then(|value|value.checked_add(1).map(|next|(value,next))).expect("SimdKey::encode exceeds usize");match self{Self::U16(positions)=>positions[value.0]!=positions[value.1],Self::U32(positions)=>positions[value.0]!=positions[value.1],}}}fn build_direct_positions<K,P>(values:&[K],bits:u32,position:impl Fn(usize)->P)->Vec<P>where K:SimdKey,P:Copy{let len=(1<<bits)+1;let mut positions=Vec::with_capacity(len);let maximum=(1u128<<bits)-1;let mut previous:Option<(K,u128)>=None;for(index,&value)in values.iter().enumerate(){let encoded=value.encode();assert!(encoded<=maximum,"SimdKey::encode exceeds its declared width");if let Some((previous_value,previous_encoded))=previous{assert_eq!(previous_value.cmp(&value),previous_encoded.cmp(&encoded),"SimdKey::encode does not preserve order");}if previous.is_none_or(|(_,previous)|previous!=encoded){positions.resize(encoded as usize+1,position(index));}previous=Some((value,encoded));}positions.resize(len,position(values.len()));positions}#[repr(C,align(64))]#[derive(Clone,Debug)]struct SearchBlock<T,const B:usize>([T;B]);#[derive(Clone,Debug)]struct StaticSearchTree<T,const B:usize>{values:Vec<SearchBlock<T,B>>,len:usize,maximum:T,levels:Vec<Vec<SearchBlock<T,B>>>,#[cfg(target_arch="x86_64")]backend:SimdBackend}impl<T:Copy+Ord,const B:usize>StaticSearchTree<T,B>{fn build<K>(values:&[K],sentinel:T,maximum_encoded:u128,convert:impl Fn(u128)->T,backend:SimdBackend)->Self where K:SimdKey{let _=&backend;let len=values.len();let mut previous:Option<(K,T)>=None;let mut separators=Vec::with_capacity(values.len().div_ceil(B));let mut blocks=Vec::with_capacity(separators.capacity());for chunk in values.chunks(B){let mut block=[sentinel;B];for(index,&value)in chunk.iter().enumerate(){let encoded=value.encode();assert!(encoded<=maximum_encoded,"SimdKey::encode exceeds its declared width");let encoded=convert(encoded);if let Some((previous_value,previous_encoded))=previous{assert!(previous_value.cmp(&value)==previous_encoded.cmp(&encoded),"SimdKey::encode does not preserve order");}previous=Some((value,encoded));block[index]=encoded;}separators.push(block[chunk.len()-1]);blocks.push(SearchBlock(block));}let maximum=separators.last().copied().unwrap_or(sentinel);let mut levels=Vec::new();while separators.len()>1{let mut blocks=Vec::with_capacity(separators.len().div_ceil(B));let mut next=Vec::with_capacity(blocks.capacity());for chunk in separators.chunks(B){let mut block=[sentinel;B];block[..chunk.len()].copy_from_slice(chunk);blocks.push(SearchBlock(block));next.push(chunk[chunk.len()-1]);}levels.push(blocks);separators=next;}Self{values:blocks,len,maximum,levels,#[cfg(target_arch="x86_64")]backend}}#[inline(always)]fn descend<F>(&self,value:T,mut position:F)->usize where F:FnMut(&[T;B],T)->usize{let mut block=0;for level in self.levels.iter().rev(){let values=&unsafe{level.get_unchecked(block)}.0;block=block*B+position(values,value);}let values=&unsafe{self.values.get_unchecked(block)}.0;(block*B+position(values,value)).min(self.len)}#[inline(always)]fn get(&self,index:usize)->T{unsafe{*self.values.get_unchecked(index/B).0 .get_unchecked(index%B)}}#[inline(always)]fn descend_batch<F>(&self,values:&[T;16],mut position:F)->[usize;16]where F:FnMut(&[T;B],T)->usize{let mut blocks=[0;16];for level in self.levels.iter().rev(){for index in 0..16{let block_values=&unsafe{level.get_unchecked(blocks[index])}.0;blocks[index]=blocks[index]*B+position(block_values,values[index]);}}for index in 0..16{let block=blocks[index];let block_values=&unsafe{self.values.get_unchecked(block)}.0;blocks[index]=(block*B+position(block_values,values[index])).min(self.len);}blocks}#[inline(always)]fn position_bisect_scalar(&self,value:T)->usize{self.descend(value,|values,value|{values.partition_point(|&current|current<value)})}#[inline(always)]fn rposition_bisect_scalar(&self,value:T)->usize{self.descend(value,|values,value|{values.partition_point(|&current|current<=value)})}#[inline(always)]fn position_bisect_batch_scalar(&self,values:&[T;16])->[usize;16]{self.descend_batch(values,|values,value|{values.partition_point(|&current|current<value)})}#[inline(always)]fn rposition_bisect_batch_scalar(&self,values:&[T;16])->[usize;16]{self.descend_batch(values,|values,value|{values.partition_point(|&current|current<=value)})}}macro_rules!impl_static_search_tree{($value:ty,$branch:expr,$first_ge_avx2:ident,$first_gt_avx2:ident,$first_ge_avx512:ident,$first_gt_avx512:ident,$avx512_features:literal)=>{impl StaticSearchTree<$value,$branch>{#[inline]fn position_bisect(&self,value:$value)->usize{if self.len==0||value>self.maximum{return self.len;}#[cfg(target_arch="x86_64")]return match self.backend{SimdBackend::Scalar=>self.position_bisect_scalar(value),SimdBackend::Avx2=>unsafe{self.position_bisect_avx2(value)},SimdBackend::Avx512=>unsafe{self.position_bisect_avx512(value)},};#[cfg(not(target_arch="x86_64"))]self.position_bisect_scalar(value)}#[inline]fn rposition_bisect(&self,value:$value)->usize{if self.len==0{return 0;}if value>=self.maximum{return self.len;}#[cfg(target_arch="x86_64")]return match self.backend{SimdBackend::Scalar=>self.rposition_bisect_scalar(value),SimdBackend::Avx2=>unsafe{self.rposition_bisect_avx2(value)},SimdBackend::Avx512=>unsafe{self.rposition_bisect_avx512(value)},};#[cfg(not(target_arch="x86_64"))]self.rposition_bisect_scalar(value)}#[inline]fn contains(&self,value:$value)->bool{let index=self.position_bisect(value);index<self.len&&self.get(index)==value}#[inline]fn position_bisect_batch(&self,values:&[$value;16])->[usize;16]{if self.len==0{return[0;16];}let mut values=*values;let mut beyond=[false;16];for index in 0..16{beyond[index]=values[index]>self.maximum;values[index]=values[index].min(self.maximum);}#[cfg(target_arch="x86_64")]let mut result=match self.backend{SimdBackend::Scalar=>self.position_bisect_batch_scalar(&values),SimdBackend::Avx2=>unsafe{self.position_bisect_batch_avx2(&values)},SimdBackend::Avx512=>unsafe{self.position_bisect_batch_avx512(&values)},};#[cfg(not(target_arch="x86_64"))]let mut result=self.position_bisect_batch_scalar(&values);for index in 0..16{if beyond[index]{result[index]=self.len;}}result}#[inline]fn rposition_bisect_batch(&self,values:&[$value;16])->[usize;16]{if self.len==0{return[0;16];}let mut values=*values;let mut beyond=[false;16];for index in 0..16{beyond[index]=values[index]>=self.maximum;}let Some(value)=values.iter().copied().find(|&value|value<self.maximum)else{return[self.len;16];};for index in 0..16{if beyond[index]{values[index]=value;}}#[cfg(target_arch="x86_64")]let mut result=match self.backend{SimdBackend::Scalar=>self.rposition_bisect_batch_scalar(&values),SimdBackend::Avx2=>unsafe{self.rposition_bisect_batch_avx2(&values)},SimdBackend::Avx512=>unsafe{self.rposition_bisect_batch_avx512(&values)},};#[cfg(not(target_arch="x86_64"))]let mut result=self.rposition_bisect_batch_scalar(&values);for index in 0..16{if beyond[index]{result[index]=self.len;}}result}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]unsafe fn position_bisect_avx2(&self,value:$value)->usize{self.descend(value,|values,value|unsafe{simd::$first_ge_avx2(values,value)})}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]unsafe fn rposition_bisect_avx2(&self,value:$value)->usize{self.descend(value,|values,value|unsafe{simd::$first_gt_avx2(values,value)})}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]unsafe fn position_bisect_batch_avx2(&self,values:&[$value;16])->[usize;16]{self.descend_batch(values,|values,value|unsafe{simd::$first_ge_avx2(values,value)})}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]unsafe fn rposition_bisect_batch_avx2(&self,values:&[$value;16])->[usize;16]{self.descend_batch(values,|values,value|unsafe{simd::$first_gt_avx2(values,value)})}#[cfg(target_arch="x86_64")]#[target_feature(enable=$avx512_features)]unsafe fn position_bisect_avx512(&self,value:$value)->usize{self.descend(value,|values,value|unsafe{simd::$first_ge_avx512(values,value)})}#[cfg(target_arch="x86_64")]#[target_feature(enable=$avx512_features)]unsafe fn rposition_bisect_avx512(&self,value:$value)->usize{self.descend(value,|values,value|unsafe{simd::$first_gt_avx512(values,value)})}#[cfg(target_arch="x86_64")]#[target_feature(enable=$avx512_features)]unsafe fn position_bisect_batch_avx512(&self,values:&[$value;16])->[usize;16]{self.descend_batch(values,|values,value|unsafe{simd::$first_ge_avx512(values,value)})}#[cfg(target_arch="x86_64")]#[target_feature(enable=$avx512_features)]unsafe fn rposition_bisect_batch_avx512(&self,values:&[$value;16])->[usize;16]{self.descend_batch(values,|values,value|unsafe{simd::$first_gt_avx512(values,value)})}}};}impl_static_search_tree!(u16,32,first_ge_u16x32_avx2,first_gt_u16x32_avx2,first_ge_u16x32_avx512,first_gt_u16x32_avx512,"avx512f,avx512bw");impl_static_search_tree!(u32,16,first_ge_u32x16_avx2,first_gt_u32x16_avx2,first_ge_u32x16_avx512,first_gt_u32x16_avx512,"avx512f");impl_static_search_tree!(u64,8,first_ge_u64x8_avx2,first_gt_u64x8_avx2,first_ge_u64x8_avx512,first_gt_u64x8_avx512,"avx512f");impl StaticSearchTree<u128,4>{#[inline]fn position_bisect(&self,value:u128)->usize{if self.len==0||value>self.maximum{self.len}else{self.descend(value,|values,value|{(values[0]<value)as usize+(values[1]<value)as usize+(values[2]<value)as usize+(values[3]<value)as usize})}}#[inline]fn rposition_bisect(&self,value:u128)->usize{if self.len==0||value>=self.maximum{self.len}else{self.descend(value,|values,value|{(values[0]<=value)as usize+(values[1]<=value)as usize+(values[2]<=value)as usize+(values[3]<=value)as usize})}}#[inline]fn contains(&self,value:u128)->bool{let index=self.position_bisect(value);index<self.len&&self.get(index)==value}#[inline]fn position_bisect_batch(&self,values:&[u128;16])->[usize;16]{if self.len==0{return[0;16];}let mut values=*values;let mut beyond=[false;16];for index in 0..16{beyond[index]=values[index]>self.maximum;values[index]=values[index].min(self.maximum);}let mut result=self.descend_batch(&values,|values,value|{(values[0]<value)as usize+(values[1]<value)as usize+(values[2]<value)as usize+(values[3]<value)as usize});for index in 0..16{if beyond[index]{result[index]=self.len;}}result}#[inline]fn rposition_bisect_batch(&self,values:&[u128;16])->[usize;16]{if self.len==0{return[0;16];}let mut values=*values;let mut beyond=[false;16];for index in 0..16{beyond[index]=values[index]>=self.maximum;}let Some(value)=values.iter().copied().find(|&value|value<self.maximum)else{return[self.len;16];};for index in 0..16{if beyond[index]{values[index]=value;}}let mut result=self.descend_batch(&values,|values,value|{(values[0]<=value)as usize+(values[1]<=value)as usize+(values[2]<=value)as usize+(values[3]<=value)as usize});for index in 0..16{if beyond[index]{result[index]=self.len;}}result}}fn search_batch<K,T>(values:&[K],output:&mut[usize],convert:impl Fn(u128)->T,single:impl Fn(T)->usize,batch:impl Fn(&[T;16])->[usize;16])where K:SimdKey,T:Copy{let mut offset=0;while offset+16<=values.len(){let values=std::array::from_fn(|index|convert(values[offset+index].encode()));output[offset..offset+16].copy_from_slice(&batch(&values));offset+=16;}let remaining=values.len()-offset;if remaining>=8{let mut encoded=[convert(values[offset].encode());16];for index in 1..remaining{encoded[index]=convert(values[offset+index].encode());}let positions=batch(&encoded);output[offset..].copy_from_slice(&positions[..remaining]);}else{for(&value,position)in values[offset..].iter().zip(&mut output[offset..]){*position=single(convert(value.encode()));}}}#[derive(Clone,Debug)]enum StaticSearchStorage{Direct(DirectStaticSearch),U16(StaticSearchTree<u16,32>),U32(StaticSearchTree<u32,16>),U64(StaticSearchTree<u64,8>),U128(StaticSearchTree<u128,4>)}impl StaticSearchStorage{#[inline(always)]fn position_bisect(&self,value:u128)->usize{match self{Self::Direct(search)=>search.position_bisect(value),Self::U16(search)=>search.position_bisect(u16::try_from(value).expect("SimdKey::encode exceeds its declared width")),Self::U32(search)=>search.position_bisect(u32::try_from(value).expect("SimdKey::encode exceeds its declared width")),Self::U64(search)=>search.position_bisect(u64::try_from(value).expect("SimdKey::encode exceeds its declared width")),Self::U128(search)=>search.position_bisect(value),}}#[inline(always)]fn rposition_bisect(&self,value:u128)->usize{match self{Self::Direct(search)=>search.rposition_bisect(value),Self::U16(search)=>search.rposition_bisect(u16::try_from(value).expect("SimdKey::encode exceeds its declared width")),Self::U32(search)=>search.rposition_bisect(u32::try_from(value).expect("SimdKey::encode exceeds its declared width")),Self::U64(search)=>search.rposition_bisect(u64::try_from(value).expect("SimdKey::encode exceeds its declared width")),Self::U128(search)=>search.rposition_bisect(value),}}#[inline(always)]fn contains(&self,value:u128)->bool{match self{Self::Direct(search)=>search.contains(value),Self::U16(search)=>search.contains(u16::try_from(value).expect("SimdKey::encode exceeds its declared width")),Self::U32(search)=>search.contains(u32::try_from(value).expect("SimdKey::encode exceeds its declared width")),Self::U64(search)=>search.contains(u64::try_from(value).expect("SimdKey::encode exceeds its declared width")),Self::U128(search)=>search.contains(value),}}fn position_bisect_batch<K:SimdKey>(&self,values:&[K],output:&mut[usize]){match self{Self::Direct(search)=>{for(&value,position)in values.iter().zip(output){*position=search.position_bisect(value.encode());}}Self::U16(search)=>search_batch(values,output,|value|u16::try_from(value).expect("SimdKey::encode exceeds its declared width"),|value|search.position_bisect(value),|values|search.position_bisect_batch(values)),Self::U32(search)=>search_batch(values,output,|value|u32::try_from(value).expect("SimdKey::encode exceeds its declared width"),|value|search.position_bisect(value),|values|search.position_bisect_batch(values)),Self::U64(search)=>search_batch(values,output,|value|u64::try_from(value).expect("SimdKey::encode exceeds its declared width"),|value|search.position_bisect(value),|values|search.position_bisect_batch(values)),Self::U128(search)=>search_batch(values,output,|value|value,|value|search.position_bisect(value),|values|search.position_bisect_batch(values)),}}fn rposition_bisect_batch<K:SimdKey>(&self,values:&[K],output:&mut[usize]){match self{Self::Direct(search)=>{for(&value,position)in values.iter().zip(output){*position=search.rposition_bisect(value.encode());}}Self::U16(search)=>search_batch(values,output,|value|u16::try_from(value).expect("SimdKey::encode exceeds its declared width"),|value|search.rposition_bisect(value),|values|search.rposition_bisect_batch(values)),Self::U32(search)=>search_batch(values,output,|value|u32::try_from(value).expect("SimdKey::encode exceeds its declared width"),|value|search.rposition_bisect(value),|values|search.rposition_bisect_batch(values)),Self::U64(search)=>search_batch(values,output,|value|u64::try_from(value).expect("SimdKey::encode exceeds its declared width"),|value|search.rposition_bisect(value),|values|search.rposition_bisect_batch(values)),Self::U128(search)=>search_batch(values,output,|value|value,|value|search.rposition_bisect(value),|values|search.rposition_bisect_batch(values)),}}}#[doc=" A static search index over sorted integer or integer-encoded keys."]#[doc=""]#[doc=" The index adds build time and storage. For a small number of searches, search the sorted slice"]#[doc=" directly instead."]#[derive(Clone,Debug)]pub struct StaticSearch<K>{storage:StaticSearchStorage,len:usize,marker:PhantomData<fn()->K>}impl<K:SimdKey>StaticSearch<K>{pub fn benchmark_from_sorted_with_backend(values:&[K],backend:SimdBackend)->Self{Self::build(values,backend,false)}#[doc=" Builds an index over sorted `values`."]#[doc=""]#[doc=" # Panics"]#[doc=""]#[doc=" Panics if `values` is not sorted or `SimdKey` does not satisfy its contract."]pub fn from_sorted(values:&[K])->Self{Self::build(values,static_search_backend(K::BITS),false)}#[doc=" Builds a direct lookup table over sorted 8-bit or 16-bit `values`."]#[doc=""]#[doc=" This layout uses a fixed table of 257 or 65,537 positions and is intended"]#[doc=" for query-heavy workloads."]#[doc=""]#[doc=" # Panics"]#[doc=""]#[doc=" Panics if `K::BITS` is neither 8 nor 16, or if `values` is not sorted."]pub fn from_sorted_direct(values:&[K])->Self{assert!(matches!(K::BITS,8|16));Self::build(values,static_search_backend(K::BITS),true)}#[inline]pub fn len(&self)->usize{self.len}#[inline]pub fn is_empty(&self)->bool{self.len==0}#[doc=" Returns the first index whose value is greater than or equal to `value`."]#[inline]pub fn position_bisect(&self,value:K)->usize{self.storage.position_bisect(value.encode())}#[doc=" Returns one past the last index whose value is less than or equal to `value`."]#[inline]pub fn rposition_bisect(&self,value:K)->usize{self.storage.rposition_bisect(value.encode())}#[doc=" Writes the first index greater than or equal to each value into `output`."]#[doc=""]#[doc=" # Panics"]#[doc=""]#[doc=" Panics if `values` and `output` have different lengths."]pub fn position_bisect_batch(&self,values:&[K],output:&mut[usize]){assert_eq!(values.len(),output.len());self.storage.position_bisect_batch(values,output);}#[doc=" Writes one past the last index less than or equal to each value into `output`."]#[doc=""]#[doc=" # Panics"]#[doc=""]#[doc=" Panics if `values` and `output` have different lengths."]pub fn rposition_bisect_batch(&self,values:&[K],output:&mut[usize]){assert_eq!(values.len(),output.len());self.storage.rposition_bisect_batch(values,output);}#[inline]pub fn range(&self,value:K)->Range<usize>{let value=value.encode();self.storage.position_bisect(value)..self.storage.rposition_bisect(value)}#[inline]pub fn contains(&self,value:K)->bool{self.storage.contains(value.encode())}fn build(values:&[K],backend:SimdBackend,direct:bool)->Self{assert!(matches!(K::BITS,8|16|32|64|128));assert!(values.windows(2).all(|pair|pair[0]<=pair[1]));let len=values.len();let storage=match K::BITS{8=>StaticSearchStorage::Direct(DirectStaticSearch::build(values,K::BITS)),16=>{if direct{StaticSearchStorage::Direct(DirectStaticSearch::build(values,K::BITS))}else{StaticSearchStorage::U16(StaticSearchTree::build(values,u16::MAX,u16::MAX as u128,|value|value as u16,backend))}}32=>StaticSearchStorage::U32(StaticSearchTree::build(values,u32::MAX,u32::MAX as u128,|value|value as u32,backend)),64=>StaticSearchStorage::U64(StaticSearchTree::build(values,u64::MAX,u64::MAX as u128,|value|value as u64,backend)),128=>StaticSearchStorage::U128(StaticSearchTree::build(values,u128::MAX,u128::MAX,|value|value,backend)),_=>unreachable!(),};Self{storage,len,marker:PhantomData}}}}
// codesnip-guard: TupleOperation
#[cfg_attr(any(),rustfmt::skip)]mod tuple_operation_impl{use super::*;macro_rules!impl_tuple_operation{(@impl)=>{impl Magma for(){type T=();fn operate(_x:&Self::T,_y:&Self::T)->Self::T{}}impl Unital for(){fn unit()->Self::T{}}impl Associative for(){}impl Commutative for(){}impl Idempotent for(){}impl Invertible for(){fn inverse(_x:&Self::T)->Self::T{}}};(@impl$($T:ident$i:tt)*)=>{impl<$($T:Magma),*>Magma for($($T,)*){type T=($(<$T as Magma>::T,)*);fn operate(x:&Self::T,y:&Self::T)->Self::T{($(<$T as Magma>::operate(&x.$i,&y.$i),)*)}}impl<$($T:Unital),*>Unital for($($T,)*){fn unit()->Self::T{($(<$T as Unital>::unit(),)*)}}impl<$($T:Associative),*>Associative for($($T,)*){}impl<$($T:Commutative),*>Commutative for($($T,)*){}impl<$($T:Idempotent),*>Idempotent for($($T,)*){}impl<$($T:Invertible),*>Invertible for($($T,)*){fn inverse(x:&Self::T)->Self::T{($(<$T as Invertible>::inverse(&x.$i),)*)}}};(@inner$($T:ident$i:tt)*;$U:ident$j:tt$($t:tt)*)=>{impl_tuple_operation!(@impl$($T$i)*);impl_tuple_operation!(@inner$($T$i)*$U$j;$($t)*);};(@inner$($T:ident$i:tt)*;)=>{impl_tuple_operation!(@impl$($T$i)*);};($($t:tt)*)=>{impl_tuple_operation!(@inner;$($t)*);};}impl_tuple_operation!(A 0 B 1 C 2 D 3 E 4 F 5 G 6 H 7 I 8 J 9);}
// codesnip-guard: WaveletMatrix
#[cfg_attr(any(),rustfmt::skip)]pub use self::wavelet_matrix::{WaveletMatrix,WaveletMatrixPointAdd};#[cfg_attr(any(),rustfmt::skip)]mod wavelet_matrix{use super::{AbelianGroup,BinaryIndexedTree,BitVector,Compressor,RankSelectDictionaries,VecCompress};use std::{mem::{self,MaybeUninit},ops::Range};#[derive(Debug,Clone)]pub struct WaveletMatrix<T>{len:usize,bit_length:usize,zeros:Vec<usize>,bit_vectors:Vec<BitVector>,compress:VecCompress<T>}impl<T>WaveletMatrix<T>where T:Ord+Clone{pub fn benchmark_scalar(mut self)->Self{self.bit_vectors=self.bit_vectors.into_iter().map(BitVector::benchmark_scalar).collect();self}pub fn benchmark_bmi2(mut self)->Self{self.bit_vectors=self.bit_vectors.into_iter().map(BitVector::benchmark_bmi2).collect();self}pub fn new(v:Vec<T>)->Self{let len=v.len();let mut sorted:Vec<_>=v.into_iter().enumerate().map(|(i,value)|(value,i)).collect();sorted.sort_unstable_by(|a,b|a.0 .cmp(&b.0));let mut values=Vec::with_capacity(len);let mut indices=vec![0;len];for(value,i)in sorted{if values.last().is_none_or(|last|last!=&value){values.push(value);}indices[i]=values.len()-1;}let compress=VecCompress::from_sorted_unique(values);let bit_length=usize::BITS as usize-compress.size().leading_zeros()as usize;let mut bit_vectors=Vec::with_capacity(bit_length);let mut zeros=Vec::with_capacity(bit_length);let mut next=Vec::with_capacity(len);let mut ones=Vec::with_capacity(len);for d in(0..bit_length).rev(){bit_vectors.push(indices.iter().map(|&idx|((idx>>d)&1)!=0).collect());for&idx in&indices{if((idx>>d)&1)==0{next.push(idx);}else{ones.push(idx);}}zeros.push(next.len());next.append(&mut ones);mem::swap(&mut indices,&mut next);next.clear();}Self{len,bit_length,zeros,bit_vectors,compress}}pub fn new_with_init<F>(v:Vec<T>,mut f:F)->Self where F:FnMut(usize,usize,T){let this=Self::new(v.clone());for(mut k,value)in v.into_iter().enumerate(){for d in(0..this.bit_length).rev(){let level=this.level(d);let(bit,rank1)=this.bit_vectors[level].access_rank1(k);if bit{k=this.zeros[level]+rank1;}else{k-=rank1;}f(d,k,value.clone());}}this}fn level(&self,d:usize)->usize{self.bit_length-1-d}fn rank1(&self,level:usize,k:usize)->usize{self.bit_vectors[level].rank1(k)}fn rank0(&self,level:usize,k:usize)->usize{k-self.rank1(level,k)}fn reorder<U>(&self,level:usize,current:Vec<U>,mut visit:impl FnMut(&U))->Vec<U>{assert_eq!(current.len(),self.len);let mut next=Vec::with_capacity(self.len);next.resize_with(self.len,MaybeUninit::uninit);let mut zero=0;let mut one=self.zeros[level];for(i,value)in current.into_iter().enumerate(){visit(&value);if self.bit_vectors[level].access(i){next[one].write(value);one+=1;}else{next[zero].write(value);zero+=1;}}unsafe{let mut next=mem::ManuallyDrop::new(next);Vec::from_raw_parts(next.as_mut_ptr().cast(),next.len(),next.capacity())}}fn rank_by_index(&self,idx:usize,mut range:Range<usize>)->usize{for d in(0..self.bit_length).rev(){let level=self.level(d);let start1=self.rank1(level,range.start);let end1=self.rank1(level,range.end);if((idx>>d)&1)!=0{range.start=self.zeros[level]+start1;range.end=self.zeros[level]+end1;}else{range.start-=start1;range.end-=end1;}}range.end-range.start}#[doc=" get k-th value"]pub fn access(&self,mut k:usize)->T{let mut idx=0;for d in(0..self.bit_length).rev(){let level=self.level(d);let(bit,rank1)=self.bit_vectors[level].access_rank1(k);if bit{idx|=1<<d;k=self.zeros[level]+rank1;}else{k-=rank1;}}self.compress.values()[idx].clone()}#[doc=" Returns the values at `indices`, traversing queries together in groups of 16."]pub fn access_batch(&self,indices:impl IntoIterator<Item=usize>)->Vec<T>{let indices:Vec<_>=indices.into_iter().collect();if indices.len()<8{return indices.into_iter().map(|index|self.access(index)).collect();}assert!(self.len<=u32::MAX as usize);let mut result=Vec::with_capacity(indices.len());for indices in indices.chunks(16){if indices.len()<8{result.extend(indices.iter().map(|&index|self.access(index)));continue;}let mut states=[[0_u32;2];16];for(state,&index)in states.iter_mut().zip(indices){state[0]=index as u32;}for d in(0..self.bit_length).rev(){let level=self.level(d);for state in&mut states[..indices.len()]{let position=state[0]as usize;let(bit,rank1)=self.bit_vectors[level].access_rank1(position);if bit{state[0]=(self.zeros[level]+rank1)as u32;state[1]|=1<<d;}else{state[0]=(position-rank1)as u32;}}}result.extend(states[..indices.len()].iter().map(|state|self.compress.values()[state[1]as usize].clone()));}result}#[doc=" the number of val in range"]pub fn rank(&self,val:T,range:Range<usize>)->usize{match self.compress.index_exact(&val){Some(idx)=>self.rank_by_index(idx,range),None=>0,}}#[doc=" Returns the number of exact matches for each `(value, range)` query."]pub fn rank_batch(&self,queries:impl IntoIterator<Item=(T,Range<usize>)>)->Vec<usize>{let queries:Vec<_>=queries.into_iter().collect();if queries.len()<8{return queries.into_iter().map(|(value,range)|self.rank(value,range)).collect();}assert!(self.len<=u32::MAX as usize);let mut result=vec![0;queries.len()];let mut active=Vec::new();for(output,(value,range))in queries.into_iter().enumerate(){if let Some(index)=self.compress.index_exact(&value){active.push((output,index,range));}}for queries in active.chunks(16){if queries.len()<8{for(output,index,range)in queries{result[*output]=self.rank_by_index(*index,range.clone());}continue;}let mut states=[[0_u32;3];16];for(state,(_,index,range))in states.iter_mut().zip(queries){*state=[range.start as u32,range.end as u32,*index as u32];}for d in(0..self.bit_length).rev(){let level=self.level(d);for state in&mut states[..queries.len()]{let start=state[0]as usize;let end=state[1]as usize;let start1=self.rank1(level,start);let end1=self.rank1(level,end);if((state[2]>>d)&1)!=0{state[0]=(self.zeros[level]+start1)as u32;state[1]=(self.zeros[level]+end1)as u32;}else{state[0]=(start-start1)as u32;state[1]=(end-end1)as u32;}}}for((output,_,_),state)in queries.iter().zip(&states){result[*output]=(state[1]-state[0])as usize;}}result}#[doc=" index of k-th val"]pub fn select(&self,val:T,k:usize)->Option<usize>{let idx=self.compress.index_exact(&val)?;if self.rank_by_index(idx,0..self.len)<=k{return None;}let mut i=0;for d in(0..self.bit_length).rev(){let level=self.level(d);if((idx>>d)&1)!=0{i=self.zeros[level]+self.rank1(level,i);}else{i=self.rank0(level,i);}}i+=k;for level in(0..self.bit_length).rev(){if i>=self.zeros[level]{i=self.bit_vectors[level].select1(i-self.zeros[level]).unwrap();}else{i=self.bit_vectors[level].select0(i).unwrap();}}Some(i)}#[doc=" get k-th smallest value in range"]pub fn quantile(&self,mut range:Range<usize>,mut k:usize)->T{let mut idx=0;for d in(0..self.bit_length).rev(){let level=self.level(d);let start1=self.rank1(level,range.start);let end1=self.rank1(level,range.end);let start0=range.start-start1;let end0=range.end-end1;let z=end0-start0;if z<=k{k-=z;idx|=1<<d;range.start=self.zeros[level]+start1;range.end=self.zeros[level]+end1;}else{range.start=start0;range.end=end0;}}self.compress.values()[idx].clone()}pub fn quantile_batch(&self,queries:impl IntoIterator<Item=(Range<usize>,usize)>)->Vec<T>{let queries:Vec<_>=queries.into_iter().collect();if queries.len()<8{return queries.into_iter().map(|(range,k)|self.quantile(range,k)).collect();}assert!(self.len<=u32::MAX as usize);let mut result=Vec::with_capacity(queries.len());for queries in queries.chunks(16){if queries.len()<8{result.extend(queries.iter().map(|(range,k)|self.quantile(range.clone(),*k)));continue;}let mut states=[[0_u32;4];16];for(state,(range,k))in states.iter_mut().zip(queries){*state=[range.start as u32,range.end as u32,*k as u32,0];}for d in(0..self.bit_length).rev(){let level=self.level(d);for state in&mut states[..queries.len()]{let start=state[0]as usize;let end=state[1]as usize;let start1=self.rank1(level,start);let end1=self.rank1(level,end);let start0=(start-start1)as u32;let end0=(end-end1)as u32;let zeros=end0-start0;let mask=0u32.wrapping_sub((state[2]>=zeros)as u32);state[0]=(start0&!mask)|((self.zeros[level]as u32+start1 as u32)&mask);state[1]=(end0&!mask)|((self.zeros[level]as u32+end1 as u32)&mask);state[2]-=zeros&mask;state[3]|=(1u32<<d)&mask;}}result.extend(states[..queries.len()].iter().map(|state|self.compress.values()[state[3]as usize].clone()));}result}#[doc=" get k-th smallest value out of range"]pub fn quantile_outer(&self,mut range:Range<usize>,mut k:usize)->T{let mut idx=0;let mut orange=0..self.len;for d in(0..self.bit_length).rev(){let level=self.level(d);let range_start1=self.rank1(level,range.start);let range_end1=self.rank1(level,range.end);let outer_start1=self.rank1(level,orange.start);let outer_end1=self.rank1(level,orange.end);let range_start0=range.start-range_start1;let range_end0=range.end-range_end1;let outer_start0=orange.start-outer_start1;let outer_end0=orange.end-outer_end1;let z=(outer_end0-outer_start0)-(range_end0-range_start0);if z<=k{k-=z;idx|=1<<d;range.start=self.zeros[level]+range_start1;range.end=self.zeros[level]+range_end1;orange.start=self.zeros[level]+outer_start1;orange.end=self.zeros[level]+outer_end1;}else{range.start=range_start0;range.end=range_end0;orange.start=outer_start0;orange.end=outer_end0;}}self.compress.values()[idx].clone()}#[doc=" the number of value less than val in range"]pub fn rank_lessthan(&self,val:T,mut range:Range<usize>)->usize{let idx=self.compress.index_lower_bound(&val);let mut res=0;for d in(0..self.bit_length).rev(){let level=self.level(d);let start1=self.rank1(level,range.start);let end1=self.rank1(level,range.end);if((idx>>d)&1)!=0{res+=(range.end-end1)-(range.start-start1);range.start=self.zeros[level]+start1;range.end=self.zeros[level]+end1;}else{range.start-=start1;range.end-=end1;}}res}#[doc=" the number of valrange in range"]pub fn rank_range(&self,valrange:Range<T>,range:Range<usize>)->usize{self.rank_lessthan(valrange.end,range.clone())-self.rank_lessthan(valrange.start,range)}pub fn query_less_than<F>(&self,val:T,mut range:Range<usize>,mut f:F)where F:FnMut(usize,Range<usize>){let idx=self.compress.index_lower_bound(&val);for d in(0..self.bit_length).rev(){let level=self.level(d);let start1=self.rank1(level,range.start);let end1=self.rank1(level,range.end);let start0=range.start-start1;let end0=range.end-end1;if((idx>>d)&1)!=0{f(d,start0..end0);range.start=self.zeros[level]+start1;range.end=self.zeros[level]+end1;}else{range.start=start0;range.end=end0;}}}pub fn build_fold<M>(&self,weights:&[M::T])->WaveletMatrixFold<'_,T,M>where M:AbelianGroup{let len=self.len;assert_eq!(weights.len(),len);let mut prefix=Vec::with_capacity((self.bit_length+1)*(len+1));let mut current:Vec<M::T>=weights.to_vec();for level in 0..self.bit_length{let mut acc=M::unit();prefix.push(acc.clone());current=self.reorder(level,current,|w|{acc=M::operate(&acc,w);prefix.push(acc.clone());});}let mut acc=M::unit();prefix.push(acc.clone());for w in current.into_iter(){acc=M::operate(&acc,&w);prefix.push(acc.clone());}WaveletMatrixFold{wavelet_matrix:self,prefix}}pub fn build_point_add<M>(&self,weights:&[M::T])->WaveletMatrixPointAdd<'_,T,M>where M:AbelianGroup{assert_eq!(weights.len(),self.len);let mut current=weights.to_vec();let mut bits=Vec::with_capacity(self.bit_length);for level in 0..self.bit_length{current=self.reorder(level,current,|_|{});bits.push(BinaryIndexedTree::from_slice(&current));}WaveletMatrixPointAdd{wavelet_matrix:self,bits}}}pub struct WaveletMatrixPointAdd<'a,T,M>where T:Ord+Clone,M:AbelianGroup{wavelet_matrix:&'a WaveletMatrix<T>,bits:Vec<BinaryIndexedTree<M>>}impl<'a,T,M>WaveletMatrixPointAdd<'a,T,M>where T:Ord+Clone,M:AbelianGroup{pub fn update(&mut self,mut index:usize,value:M::T){debug_assert!(index<self.wavelet_matrix.len);for d in(0..self.wavelet_matrix.bit_length).rev(){let level=self.wavelet_matrix.level(d);let(bit,rank1)=self.wavelet_matrix.bit_vectors[level].access_rank1(index);if bit{index=self.wavelet_matrix.zeros[level]+rank1;}else{index-=rank1;}self.bits[level].update(index,value.clone());}}pub fn fold_lessthan(&self,value:T,range:Range<usize>)->M::T{let mut result=M::unit();self.wavelet_matrix.query_less_than(value,range,|d,range|{M::operate_assign(&mut result,&self.bits[self.wavelet_matrix.level(d)].fold(range.start,range.end));});result}pub fn fold_range(&self,values:Range<T>,range:Range<usize>)->M::T{M::rinv_operate(&self.fold_lessthan(values.end,range.clone()),&self.fold_lessthan(values.start,range))}}#[derive(Debug,Clone)]pub struct WaveletMatrixFold<'a,T,M>where T:Ord+Clone,M:AbelianGroup{wavelet_matrix:&'a WaveletMatrix<T>,prefix:Vec<M::T>}impl<'a,T,M>WaveletMatrixFold<'a,T,M>where T:Ord+Clone,M:AbelianGroup{#[inline]fn range_sum(&self,level:usize,range:Range<usize>)->M::T{let offset=level*(self.wavelet_matrix.len+1);unsafe{M::rinv_operate(self.prefix.get_unchecked(offset+range.end),self.prefix.get_unchecked(offset+range.start))}}pub fn fold_lessthan(&self,val:T,range:Range<usize>)->M::T{self.fold_lessthan_with_count(val,range).1}pub fn fold_lessthan_with_count(&self,val:T,mut range:Range<usize>)->(usize,M::T){debug_assert!(range.end<=self.wavelet_matrix.len);let idx=self.wavelet_matrix.compress.index_lower_bound(&val);let mut count=0;let mut sum=M::unit();for d in(0..self.wavelet_matrix.bit_length).rev(){let level=self.wavelet_matrix.level(d);let start0=self.wavelet_matrix.rank0(level,range.start);let end0=self.wavelet_matrix.rank0(level,range.end);if((idx>>d)&1)!=0{count+=end0-start0;sum=M::operate(&sum,&self.range_sum(level+1,start0..end0));range.start=self.wavelet_matrix.zeros[level]+(range.start-start0);range.end=self.wavelet_matrix.zeros[level]+(range.end-end0);}else{range.start=start0;range.end=end0;}}(count,sum)}pub fn fold_range(&self,valrange:Range<T>,range:Range<usize>)->M::T{M::rinv_operate(&self.fold_lessthan(valrange.end,range.clone()),&self.fold_lessthan(valrange.start,range))}pub fn fold_range_with_count(&self,valrange:Range<T>,range:Range<usize>)->(usize,M::T){let(count_upper,sum_upper)=self.fold_lessthan_with_count(valrange.end,range.clone());let(count_lower,sum_lower)=self.fold_lessthan_with_count(valrange.start,range);(count_upper-count_lower,M::rinv_operate(&sum_upper,&sum_lower))}}}
// codesnip-guard: WidePrefix
#[cfg_attr(any(),rustfmt::skip)]pub use self::wide_prefix::{WidePrefixU32,WidePrefixU64};#[cfg_attr(any(),rustfmt::skip)]mod wide_prefix{#[cfg(target_arch="x86_64")]use super::simd;use super::{SimdBackend,simd_backend};#[repr(C,align(64))]#[derive(Clone,Debug)]struct PrefixBlock<T,const B:usize>([T;B]);macro_rules!define_wide_prefix{($name:ident,$value:ty,$branch:expr,$add_avx2:ident,$first_gt_avx2:ident,$add_avx512:ident,$first_gt_avx512:ident)=>{#[doc=" A cache-line-oriented prefix-sum structure for point updates and prefix searches."]#[doc=""]#[doc=" `BinaryIndexedTree` has a denser layout and can be preferable for small workloads."]#[doc=" Use this type for repeated updates and prefix searches, especially with SIMD support."]#[derive(Clone,Debug)]pub struct$name{levels:Vec<Vec<PrefixBlock<$value,$branch>>>,len:usize,total:$value,partition_valid:bool,#[cfg(target_arch="x86_64")]backend:SimdBackend,}impl$name{pub fn new(len:usize)->Self{Self::zeroed(len,simd_backend())}pub fn from_slice(values:&[$value])->Self{Self::build(values,simd_backend())}pub fn benchmark_zeroed(len:usize,backend:SimdBackend)->Self{Self::zeroed(len,backend)}pub fn benchmark_build(values:&[$value],backend:SimdBackend)->Self{Self::build(values,backend)}#[inline]pub fn len(&self)->usize{self.len}#[inline]pub fn is_empty(&self)->bool{self.len==0}#[doc=" Adds `value` at `index`. Arithmetic is wrapping."]#[inline]pub fn update(&mut self,index:usize,value:$value){assert!(index<self.len);self.add(index,value);self.partition_valid&=self.total.checked_add(value).is_some();self.total=self.total.wrapping_add(value);}#[doc=" Replaces the value at `index`. Arithmetic is wrapping."]#[inline]pub fn set(&mut self,index:usize,value:$value){let previous=self.get(index);self.add(index,value.wrapping_sub(previous));self.partition_valid&=self.total.checked_sub(previous).and_then(|total|total.checked_add(value)).is_some();self.total=self.total.wrapping_sub(previous).wrapping_add(value);}#[doc=" Returns the wrapping sum of `0..end`."]#[inline]pub fn accumulate0(&self,mut end:usize)->$value{assert!(end<=self.len);if end==self.len{return self.total;}let mut result:$value=0;for level in&self.levels{let block=end/$branch;let lane=end%$branch;if lane!=0{let value=unsafe{level.get_unchecked(block).0 .get_unchecked(lane-1)};result=result.wrapping_add(*value);}end=block;}result}#[doc=" Returns the wrapping sum of `0..=index`."]#[inline]pub fn accumulate(&self,index:usize)->$value{self.accumulate0(index+1)}#[doc=" Returns the wrapping sum of `left..right`."]#[inline]pub fn fold(&self,left:usize,right:usize)->$value{assert!(left<=right&&right<=self.len);self.accumulate0(right).wrapping_sub(self.accumulate0(left))}#[inline]pub fn get(&self,index:usize)->$value{self.fold(index,index+1)}#[inline]pub fn fold_all(&self)->$value{self.total}#[doc=" Returns the number of leading values whose inclusive prefix sum is at most `value`."]#[doc=""]#[doc=" # Panics"]#[doc=""]#[doc=" Panics if a prefix sum has overflowed."]#[inline]pub fn partition_point_acc(&self,value:$value)->usize{assert!(self.partition_valid,"prefix sum overflowed");if value>=self.total{return self.len;}#[cfg(target_arch="x86_64")]return match self.backend{SimdBackend::Scalar=>self.partition_point_scalar(value),SimdBackend::Avx2=>unsafe{self.partition_point_avx2(value)},SimdBackend::Avx512=>unsafe{self.partition_point_avx512(value)},};#[cfg(not(target_arch="x86_64"))]self.partition_point_scalar(value)}fn zeroed(len:usize,backend:SimdBackend)->Self{let _=&backend;let mut levels=Vec::new();let mut level_len=len;while level_len!=0{level_len=level_len.div_ceil($branch);levels.push(vec![PrefixBlock([0;$branch]);level_len]);if level_len==1{break;}}Self{levels,len,total:0,partition_valid:true,#[cfg(target_arch="x86_64")]backend,}}fn build(values:&[$value],backend:SimdBackend)->Self{let _=&backend;let mut levels=Vec::new();let mut partition_valid=true;let mut current=Vec::with_capacity(values.len().div_ceil($branch));let mut blocks=Vec::with_capacity(current.capacity());for chunk in values.chunks($branch){let mut prefix=[0;$branch];let mut sum:$value=0;for(index,&value)in chunk.iter().enumerate(){partition_valid&=sum.checked_add(value).is_some();sum=sum.wrapping_add(value);prefix[index]=sum;}prefix[chunk.len()..].fill(sum);blocks.push(PrefixBlock(prefix));current.push(sum);}if!blocks.is_empty(){levels.push(blocks);}while current.len()>1{let mut blocks=Vec::with_capacity(current.len().div_ceil($branch));for chunk in current.chunks($branch){let mut prefix=[0;$branch];let mut sum:$value=0;for(index,&value)in chunk.iter().enumerate(){partition_valid&=sum.checked_add(value).is_some();sum=sum.wrapping_add(value);prefix[index]=sum;}prefix[chunk.len()..].fill(sum);blocks.push(PrefixBlock(prefix));}current=blocks.iter().map(|block|block.0[$branch-1]).collect();levels.push(blocks);}Self{levels,len:values.len(),total:current.first().copied().unwrap_or(0),partition_valid,#[cfg(target_arch="x86_64")]backend,}}#[inline]fn add(&mut self,index:usize,value:$value){#[cfg(target_arch="x86_64")]match self.backend{SimdBackend::Scalar=>self.add_scalar(index,value),SimdBackend::Avx2=>unsafe{self.add_avx2(index,value)},SimdBackend::Avx512=>unsafe{self.add_avx512(index,value)},}#[cfg(not(target_arch="x86_64"))]self.add_scalar(index,value);}#[inline]fn add_scalar(&mut self,mut index:usize,value:$value){for level in&mut self.levels{let block=index/$branch;let lane=index%$branch;let prefix=&mut unsafe{level.get_unchecked_mut(block)}.0;for prefix in&mut prefix[lane..]{*prefix=prefix.wrapping_add(value);}index=block;}}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]unsafe fn add_avx2(&mut self,mut index:usize,value:$value){for level in&mut self.levels{let block=index/$branch;let lane=index%$branch;let prefix=&mut unsafe{level.get_unchecked_mut(block)}.0;unsafe{simd::$add_avx2(prefix,lane,value)};index=block;}}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f")]unsafe fn add_avx512(&mut self,mut index:usize,value:$value){for level in&mut self.levels{let block=index/$branch;let lane=index%$branch;let prefix=&mut unsafe{level.get_unchecked_mut(block)}.0;unsafe{simd::$add_avx512(prefix,lane,value)};index=block;}}#[inline(always)]fn partition_point_by<F>(&self,mut value:$value,mut first_gt:F)->usize where F:FnMut(&[$value;$branch],$value)->usize,{let mut node=0;for level in self.levels.iter().rev(){let prefix=&unsafe{level.get_unchecked(node)}.0;let lane=first_gt(prefix,value);if lane!=0{value=value.wrapping_sub(unsafe{*prefix.get_unchecked(lane-1)});}node=node*$branch+lane;}node.min(self.len)}#[inline(always)]fn partition_point_scalar(&self,value:$value)->usize{self.partition_point_by(value,|prefix,value|{prefix.partition_point(|&sum|sum<=value)})}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]unsafe fn partition_point_avx2(&self,value:$value)->usize{self.partition_point_by(value,|prefix,value|unsafe{simd::$first_gt_avx2(prefix,value)})}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f")]unsafe fn partition_point_avx512(&self,value:$value)->usize{self.partition_point_by(value,|prefix,value|unsafe{simd::$first_gt_avx512(prefix,value)})}}};}define_wide_prefix!(WidePrefixU32,u32,16,add_suffix_u32x16_avx2,first_gt_u32x16_avx2,add_suffix_u32x16_avx512,first_gt_u32x16_avx512);define_wide_prefix!(WidePrefixU64,u64,8,add_suffix_u64x8_avx2,first_gt_u64x8_avx2,add_suffix_u64x8_avx512,first_gt_u64x8_avx512);}
// codesnip-guard: WideSegmentTree
#[cfg_attr(any(),rustfmt::skip)]pub use self::wide_segment_tree::{WideSegmentTreeMaxI32,WideSegmentTreeMaxI64,WideSegmentTreeMinI32,WideSegmentTreeMinI64};#[cfg_attr(any(),rustfmt::skip)]mod wide_segment_tree{#[cfg(target_arch="x86_64")]use super::simd;use super::{RangeBoundsExt,SimdBackend,simd_backend};use std::ops::RangeBounds;#[repr(C,align(64))]#[derive(Clone,Debug)]struct Block<T,const B:usize>([T;B]);macro_rules!define_wide_segment_tree{($name:ident,$value:ty,$branch:expr,$unit:expr,$operation:ident,$backend:expr,$reduce_avx2:ident,$reduce_range_avx2:ident,$reduce_avx512:ident,$reduce_range_avx512:ident)=>{#[doc=" A cache-line-oriented point-update segment tree for a fixed extrema operation."]#[doc=""]#[doc=" For immutable data, `RangeMinimumQuery` is faster and uses a static-only contract."]#[derive(Clone,Debug)]pub struct$name{levels:Vec<Vec<Block<$value,$branch>>>,len:usize,#[cfg(target_arch="x86_64")]backend:SimdBackend,}impl$name{pub fn new(len:usize)->Self{Self::from_vec(vec![$unit;len])}pub fn from_vec(values:Vec<$value>)->Self{Self::build(values,$backend)}pub fn benchmark_build(values:Vec<$value>,backend:SimdBackend)->Self{Self::build(values,backend)}pub fn benchmark_zeroed(len:usize,backend:SimdBackend)->Self{Self::build(vec![$unit;len],backend)}#[inline]pub fn len(&self)->usize{self.len}#[inline]pub fn is_empty(&self)->bool{self.len==0}#[inline]pub fn set(&mut self,index:usize,value:$value){assert!(index<self.len);self.set_value(index,value);}#[inline]pub fn clear(&mut self,index:usize){self.set(index,$unit);}#[inline]pub fn update(&mut self,index:usize,value:$value){assert!(index<self.len);let current=self.levels[0][index/$branch].0[index%$branch];self.set_value(index,current.$operation(value));}#[inline]pub fn get(&self,index:usize)->$value{assert!(index<self.len);self.levels[0][index/$branch].0[index%$branch]}#[inline]pub fn fold<R>(&self,range:R)->$value where R:RangeBounds<usize>,{let range=range.to_range_bounded(0,self.len).expect("invalid range");#[cfg(target_arch="x86_64")]return match self.backend{SimdBackend::Scalar=>{self.fold_by(range.start,range.end,Self::reduce_range_scalar)}SimdBackend::Avx2=>unsafe{self.fold_avx2(range.start,range.end)},SimdBackend::Avx512=>unsafe{self.fold_avx512(range.start,range.end)},};#[cfg(not(target_arch="x86_64"))]self.fold_by(range.start,range.end,Self::reduce_range_scalar)}#[inline]pub fn fold_all(&self)->$value{self.levels.last().unwrap()[0].0[0]}fn build(values:Vec<$value>,backend:SimdBackend)->Self{let _=&backend;let len=values.len();let mut current=if values.is_empty(){vec![$unit]}else{values};let mut levels=Vec::new();loop{let blocks:Vec<_> =current.chunks($branch).map(|chunk|{let mut values=[$unit;$branch];values[..chunk.len()].copy_from_slice(chunk);Block(values)}).collect();if current.len()==1{levels.push(blocks);break;}current=blocks.iter().map(|block|Self::reduce_scalar(&block.0)).collect();levels.push(blocks);}Self{levels,len,#[cfg(target_arch="x86_64")]backend,}}#[inline]fn set_value(&mut self,index:usize,value:$value){if self.levels[0][index/$branch].0[index%$branch]==value{return;}#[cfg(target_arch="x86_64")]match self.backend{SimdBackend::Scalar=>self.set_by(index,value,Self::reduce_scalar),SimdBackend::Avx2=>unsafe{self.set_avx2(index,value)},SimdBackend::Avx512=>unsafe{self.set_avx512(index,value)},}#[cfg(not(target_arch="x86_64"))]self.set_by(index,value,Self::reduce_scalar);}#[inline(always)]fn reduce_scalar(values:&[$value;$branch])->$value{let mut result=values[0];for&value in&values[1..]{result=result.$operation(value);}result}#[inline(always)]fn reduce_range_scalar(values:&[$value;$branch],start:usize,end:usize)->$value{values[start..end].iter().copied().reduce(<$value>::$operation).unwrap_or($unit)}#[inline(always)]fn set_by<F>(&mut self,mut index:usize,value:$value,mut reduce:F)where F:FnMut(&[$value;$branch])->$value,{self.levels[0][index/$branch].0[index%$branch]=value;for level in 0..self.levels.len()-1{let block=index/$branch;let aggregate=reduce(&self.levels[level][block].0);index=block;let parent=&mut self.levels[level+1][index/$branch].0[index%$branch];if*parent==aggregate{break;}*parent=aggregate;}}#[inline(always)]fn fold_by<F>(&self,mut left:usize,mut right:usize,mut reduce:F)->$value where F:FnMut(&[$value;$branch],usize,usize)->$value,{let mut result=$unit;for level in&self.levels{if left>=right{break;}let first=left/$branch;let last=(right-1)/$branch;if first==last{return result.$operation(reduce(&level[first].0,left%$branch,(right-1)%$branch+1,));}if left%$branch!=0{result=result.$operation(reduce(&level[first].0,left%$branch,$branch));left=(first+1)*$branch;}if right%$branch!=0{result=result.$operation(reduce(&level[last].0,0,right%$branch));right=last*$branch;}left/=$branch;right/=$branch;}result}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]unsafe fn set_avx2(&mut self,index:usize,value:$value){self.set_by(index,value,|values|unsafe{simd::$reduce_avx2(values)});}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]unsafe fn fold_avx2(&self,left:usize,right:usize)->$value{self.fold_by(left,right,|values,start,end|unsafe{simd::$reduce_range_avx2(values,start,end)})}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f")]unsafe fn set_avx512(&mut self,index:usize,value:$value){self.set_by(index,value,|values|unsafe{simd::$reduce_avx512(values)});}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f")]unsafe fn fold_avx512(&self,left:usize,right:usize)->$value{self.fold_by(left,right,|values,start,end|unsafe{simd::$reduce_range_avx512(values,start,end)})}}};}define_wide_segment_tree!(WideSegmentTreeMinI32,i32,16,i32::MAX,min,simd_backend(),minimum_i32x16_avx2,minimum_range_i32x16_avx2,minimum_i32x16_avx512,minimum_range_i32x16_avx512);define_wide_segment_tree!(WideSegmentTreeMaxI32,i32,16,i32::MIN,max,simd_backend(),maximum_i32x16_avx2,maximum_range_i32x16_avx2,maximum_i32x16_avx512,maximum_range_i32x16_avx512);define_wide_segment_tree!(WideSegmentTreeMinI64,i64,8,i64::MAX,min,simd_backend(),minimum_i64x8_avx2,minimum_range_i64x8_avx2,minimum_i64x8_avx512,minimum_range_i64x8_avx512);define_wide_segment_tree!(WideSegmentTreeMaxI64,i64,8,i64::MIN,max,simd_backend(),maximum_i64x8_avx2,maximum_range_i64x8_avx2,maximum_i64x8_avx512,maximum_range_i64x8_avx512);}
// codesnip-guard: algebra
#[cfg_attr(any(),rustfmt::skip)]pub use self::magma::*;#[cfg_attr(any(),rustfmt::skip)]mod magma{#![doc=" algebraic traits"]#[doc=" binary operaion: $T \\circ T \\to T$"]pub trait Magma{#[doc=" type of operands: $T$"]type T:Clone;#[doc=" binary operaion: $\\circ$"]fn operate(x:&Self::T,y:&Self::T)->Self::T;fn reverse_operate(x:&Self::T,y:&Self::T)->Self::T{Self::operate(y,x)}fn operate_assign(x:&mut Self::T,y:&Self::T){*x=Self::operate(x,y);}}#[doc=" $\\forall a,\\forall b,\\forall c \\in T, (a \\circ b) \\circ c = a \\circ (b \\circ c)$"]pub trait Associative:Magma{#[cfg(test)]fn check_associative(a:&Self::T,b:&Self::T,c:&Self::T)->bool where Self::T:PartialEq{({let ab_c=Self::operate(&Self::operate(a,b),c);let a_bc=Self::operate(a,&Self::operate(b,c));ab_c==a_bc})&&({let ab_c=Self::reverse_operate(c,&Self::reverse_operate(b,a));let a_bc=Self::reverse_operate(&Self::reverse_operate(c,b),a);ab_c==a_bc})&&({let mut ab_c=a.clone();Self::operate_assign(&mut ab_c,b);Self::operate_assign(&mut ab_c,c);let mut bc=b.clone();Self::operate_assign(&mut bc,c);let mut a_bc=a.clone();Self::operate_assign(&mut a_bc,&bc);ab_c==a_bc})}}#[doc=" associative binary operation"]pub trait SemiGroup:Magma+Associative{}impl<S>SemiGroup for S where S:Magma+Associative{}#[doc=" $\\exists e \\in T, \\forall a \\in T, e \\circ a = a \\circ e = e$"]pub trait Unital:Magma{#[doc=" identity element: $e$"]fn unit()->Self::T;fn is_unit(x:&Self::T)->bool where Self::T:PartialEq{x==&Self::unit()}fn set_unit(x:&mut Self::T){*x=Self::unit();}#[cfg(test)]fn check_unital(x:&Self::T)->bool where Self::T:PartialEq{let u=Self::unit();let xu=Self::operate(x,&u);let ux=Self::operate(&u,x);let mut any=x.clone();Self::set_unit(&mut any);xu==*x&&ux==*x&&Self::is_unit(&u)&&Self::is_unit(&any)}}pub trait ExpBits{type Iter:Iterator<Item=bool>;fn bits(self)->Self::Iter;}pub trait SignedExpBits{type T:ExpBits;fn neg_and_bits(self)->(bool,Self::T);}pub struct Bits<T>{n:T}macro_rules!impl_exp_bits_for_uint{($($t:ty)*)=>{$(impl Iterator for Bits<$t>{type Item=bool;fn next(&mut self)->Option<bool>{if self.n==0{None}else{let bit=(self.n&1)==1;self.n>>=1;Some(bit)}}}impl ExpBits for$t{type Iter=Bits<$t>;fn bits(self)->Self::Iter{Bits{n:self}}}impl SignedExpBits for$t{type T=$t;fn neg_and_bits(self)->(bool,Self::T){(false,self)}})*};}impl_exp_bits_for_uint!(u8 u16 u32 u64 u128 usize);macro_rules!impl_signed_exp_bits_for_sint{($($s:ty,$u:ty;)*)=>{$(impl SignedExpBits for$s{type T=$u;fn neg_and_bits(self)->(bool,Self::T){(self<0,self.unsigned_abs())}})*};}impl_signed_exp_bits_for_sint!(i8,u8;i16,u16;i32,u32;i64,u64;i128,u128;isize,usize;);#[doc=" associative binary operation and an identity element"]pub trait Monoid:SemiGroup+Unital{#[doc=" binary exponentiation: $x^n = x\\circ\\ddots\\circ x$"]fn pow<E>(mut x:Self::T,exp:E)->Self::T where E:ExpBits{let mut res=Self::unit();for bit in exp.bits(){if bit{res=Self::operate(&res,&x);}x=Self::operate(&x,&x);}res}fn fold<I>(iter:I)->Self::T where I:IntoIterator<Item=Self::T>{let mut iter=iter.into_iter();if let Some(item)=iter.next(){iter.fold(item,|acc,x|Self::operate(&acc,&x))}else{Self::unit()}}}impl<M>Monoid for M where M:SemiGroup+Unital{}#[doc=" $\\exists e \\in T, \\forall a \\in T, \\exists b,c \\in T, b \\circ a = a \\circ c = e$"]pub trait Invertible:Magma+Unital{#[doc=" $a$ where $a \\circ x = e$"]fn inverse(x:&Self::T)->Self::T;fn rinv_operate(x:&Self::T,y:&Self::T)->Self::T{Self::operate(x,&Self::inverse(y))}fn rinv_operate_assign(x:&mut Self::T,y:&Self::T){*x=Self::rinv_operate(x,y);}#[cfg(test)]fn check_invertible(x:&Self::T)->bool where Self::T:PartialEq{let i=Self::inverse(x);({let xi=Self::operate(x,&i);let ix=Self::operate(&i,x);Self::is_unit(&xi)&&Self::is_unit(&ix)})&&({let ii=Self::inverse(&i);ii==*x})&&({let mut xi=x.clone();Self::operate_assign(&mut xi,&i);let mut ix=i.clone();Self::operate_assign(&mut ix,x);Self::is_unit(&xi)&&Self::is_unit(&ix)})&&({let mut xi=x.clone();Self::rinv_operate_assign(&mut xi,x);let mut ix=i.clone();Self::rinv_operate_assign(&mut ix,&i);Self::is_unit(&xi)&&Self::is_unit(&ix)})}}#[doc=" associative binary operation and an identity element and inverse elements"]pub trait Group:Monoid+Invertible{fn signed_pow<E>(x:Self::T,exp:E)->Self::T where E:SignedExpBits{let(neg,exp)=E::neg_and_bits(exp);let res=Self::pow(x,exp);if neg{Self::inverse(&res)}else{res}}}impl<G>Group for G where G:Monoid+Invertible{}#[doc=" $\\forall a,\\forall b \\in T, a \\circ b = b \\circ a$"]pub trait Commutative:Magma{#[cfg(test)]fn check_commutative(a:&Self::T,b:&Self::T)->bool where Self::T:PartialEq{Self::operate(a,b)==Self::operate(b,a)}}#[doc=" commutative monoid"]pub trait AbelianMonoid:Monoid+Commutative{}impl<M>AbelianMonoid for M where M:Monoid+Commutative{}#[doc=" commutative group"]pub trait AbelianGroup:Group+Commutative{}impl<G>AbelianGroup for G where G:Group+Commutative{}#[doc=" $\\forall a \\in T, a \\circ a = a$"]pub trait Idempotent:Magma{#[cfg(test)]fn check_idempotent(a:&Self::T)->bool where Self::T:PartialEq{Self::operate(a,a)==*a}}#[doc=" idempotent monoid"]pub trait IdempotentMonoid:Monoid+Idempotent{}impl<M>IdempotentMonoid for M where M:Monoid+Idempotent{}#[macro_export]macro_rules!monoid_fold{($m:ty)=>{<$m as Unital>::unit()};($m:ty,)=>{<$m as Unital>::unit()};($m:ty,$f:expr)=>{$f};($m:ty,$f:expr,$($ff:expr),*)=>{<$m as Magma>::operate(&($f),&monoid_fold!($m,$($ff),*))};}#[macro_export]macro_rules!define_monoid{($Name:ident,$t:ty,|$x:ident,$y:ident|$op:expr,$unit:expr)=>{struct$Name;impl Magma for$Name{type T=$t;fn operate($x:&Self::T,$y:&Self::T)->Self::T{$op}}impl Unital for$Name{fn unit()->Self::T{$unit}}impl Associative for$Name{}};}}
// codesnip-guard: avx_helper
#[cfg_attr(any(),rustfmt::skip)]pub use self::avx_helper::{SimdBackend,avx512_enabled,avx512_supported,disable_avx512,enable_avx512,simd_backend};#[cfg_attr(any(),rustfmt::skip)]mod avx_helper{use std::sync::atomic::{AtomicBool,Ordering};#[derive(Copy,Clone,Debug,Eq,PartialEq)]pub enum SimdBackend{Scalar,Avx2,Avx512}static AVX512_ENABLED:AtomicBool=AtomicBool::new(true);pub fn disable_avx512(){AVX512_ENABLED.store(false,Ordering::Relaxed);}pub fn enable_avx512(){AVX512_ENABLED.store(true,Ordering::Relaxed);}pub fn avx512_enabled()->bool{AVX512_ENABLED.load(Ordering::Relaxed)}pub fn avx512_supported()->bool{#[cfg(any(target_arch="x86",target_arch="x86_64"))]return is_x86_feature_detected!("avx512f")&&is_x86_feature_detected!("avx512dq")&&is_x86_feature_detected!("avx512cd")&&is_x86_feature_detected!("avx512bw")&&is_x86_feature_detected!("avx512vl");#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]false}pub fn simd_backend()->SimdBackend{#[cfg(any(target_arch="x86",target_arch="x86_64"))]{if avx512_enabled()&&avx512_supported(){return SimdBackend::Avx512;}if is_x86_feature_detected!("avx2"){return SimdBackend::Avx2;}}SimdBackend::Scalar}#[macro_export]macro_rules!avx_helper{(@dispatch$backend:path,$kind:ident;$avx512:expr,$avx2:expr,$scalar:expr)=>{{#[cfg(target_arch="x86_64")]{match$backend(){$kind::Avx512=>$avx512,$kind::Avx2=>$avx2,$kind::Scalar=>$scalar,}}#[cfg(not(target_arch="x86_64"))]$scalar}};(@dispatch_avx2_fma$avx2:expr,$scalar:expr)=>{{#[cfg(target_arch="x86_64")]{if is_x86_feature_detected!("avx2")&&is_x86_feature_detected!("fma"){$avx2}else{$scalar}}#[cfg(not(target_arch="x86_64"))]$scalar}};(@avx512$(#[$meta:meta])*$vis:vis fn$name:ident$(<$($T:ident),+>)?($($i:ident:$t:ty),*)->$ret:ty where[$($clauses:tt)*]$body:block)=>{$(#[$meta])*$vis fn$name$(<$($T)*>)?($($i:$t),*)->$ret where$($clauses)*{if$crate::avx512_supported(){$crate::avx_helper!(@def_avx512 fn avx512$(<$($T)*>)?($($i:$t),*)->$ret where[$($clauses)*]$body);unsafe{avx512$(::<$($T),*>)?($($i),*)}}else if is_x86_feature_detected!("avx2"){$crate::avx_helper!(@def_avx2 fn avx2$(<$($T)*>)?($($i:$t),*)->$ret where[$($clauses)*]$body);unsafe{avx2$(::<$($T),*>)?($($i),*)}}else{$body}}};(@avx2$(#[$meta:meta])*$vis:vis fn$name:ident$(<$($T:ident),+>)?($($i:ident:$t:ty),*)->$ret:ty where[$($clauses:tt)*]$body:block)=>{$(#[$meta])*$vis fn$name$(<$($T)*>)?($($i:$t),*)->$ret where$($clauses)*{if is_x86_feature_detected!("avx2"){$crate::avx_helper!(@def_avx2 fn avx2$(<$($T)*>)?($($i:$t),*)->$ret where[$($clauses)*]$body);unsafe{avx2$(::<$($T),*>)?($($i),*)}}else{$body}}};(@def_avx512 fn$name:ident$(<$($T:ident),+>)?($($args:tt)*)->$ret:ty where[$($clauses:tt)*]$body:block)=>{#[target_feature(enable="avx512f,avx512dq,avx512cd,avx512bw,avx512vl")]unsafe fn$name$(<$($T)*>)?($($args)*)->$ret where$($clauses)*$body};(@def_avx2 fn$name:ident$(<$($T:ident),+>)?($($args:tt)*)->$ret:ty where[$($clauses:tt)*]$body:block)=>{#[target_feature(enable="avx2")]unsafe fn$name$(<$($T)*>)?($($args)*)->$ret where$($clauses)*$body};(@$tag:ident$(#[$meta:meta])*$vis:vis fn$name:ident$(<$($T:ident),+>)?($($args:tt)*)->$ret:ty$body:block)=>{$crate::avx_helper!(@$tag$(#[$meta])*$vis fn$name$(<$($T)*>)?($($args)*)->$ret where[]$body);};(@$tag:ident$(#[$meta:meta])*$vis:vis fn$name:ident$(<$($T:ident),+>)?($($args:tt)*)$($t:tt)*)=>{$crate::avx_helper!(@$tag$(#[$meta])*$vis fn$name$(<$($T)*>)?($($args)*)->()$($t)*);};($($t:tt)*)=>{::std::compile_error!($($t)*);}}}
// codesnip-guard: binary_search
#[cfg_attr(any(),rustfmt::skip)]pub use self::binary_search::{Bisect,SliceBisectExt,binary_search,parallel_binary_search};#[cfg_attr(any(),rustfmt::skip)]mod binary_search{use std::cmp::Ordering;#[doc=" binary search helper"]pub trait Bisect:Clone{#[doc=" Return between two elements if search is not end."]fn bisect_middle_point(&self,other:&Self)->Option<Self>;}macro_rules!impl_bisect_unsigned{($($t:ty)*)=>{$(impl Bisect for$t{fn bisect_middle_point(&self,other:&Self)->Option<Self>{if self.abs_diff(*other)>1{Some(self.midpoint(*other))}else{None}}})*};}macro_rules!impl_bisect_signed{($($t:ty)*)=>{$(impl Bisect for$t{fn bisect_middle_point(&self,other:&Self)->Option<Self>{if self.signum()!=other.signum(){if match self.cmp(other){Ordering::Less=>self+1<*other,Ordering::Equal=>false,Ordering::Greater=>other+1<*self,}{Some((*self).midpoint(*other))}else{None}}else{if self.abs_diff(*other)>1{Some(self.midpoint(*other))}else{None}}}})*};}macro_rules!impl_bisect_float{($({$t:ident$u:ident$i:ident$e:expr})*)=>{$(impl Bisect for$t{fn bisect_middle_point(&self,other:&Self)->Option<Self>{fn to_float_ord(x:$t)->$i{let a=x.to_bits()as$i;a^(((a>>$e)as$u)>>1)as$i}fn from_float_ord(a:$i)->$t{$t::from_bits((a^(((a>>$e)as$u)>>1)as$i)as _)}<$i as Bisect>::bisect_middle_point(&to_float_ord(*self),&to_float_ord(*other)).map(from_float_ord)}})*};}impl_bisect_unsigned!(u8 u16 u32 u64 u128 usize);impl_bisect_signed!(i8 i16 i32 i64 i128 isize);impl_bisect_float!({f32 u32 i32 31}{f64 u64 i64 63});#[doc=" binary search for monotone segment"]#[doc=""]#[doc=" if `ok < err` then search [ok, err) where t(`ok`), t, t, .... t, t(`ret`), f,  ... f, f, f, `err`"]#[doc=""]#[doc=" if `err < ok` then search (err, ok] where `err`, f, f, f, ... f, t(`ret`), ... t, t, t(`ok`)"]pub fn binary_search<T,F>(mut f:F,mut ok:T,mut err:T)->T where T:Bisect,F:FnMut(&T)->bool{while let Some(m)=ok.bisect_middle_point(&err){if f(&m){ok=m;}else{err=m;}}ok}#[doc=" binary search for slice"]pub trait SliceBisectExt<T>{#[doc=" Returns the first element that satisfies a predicate."]fn find_bisect(&self,f:impl FnMut(&T)->bool)->Option<&T>;#[doc=" Returns the last element that satisfies a predicate."]fn rfind_bisect(&self,f:impl FnMut(&T)->bool)->Option<&T>;#[doc=" Returns the first index that satisfies a predicate."]#[doc=" if not found, returns `len()`."]fn position_bisect(&self,f:impl FnMut(&T)->bool)->usize;#[doc=" Returns the last index+1 that satisfies a predicate."]#[doc=" if not found, returns `0`."]fn rposition_bisect(&self,f:impl FnMut(&T)->bool)->usize;}impl<T>SliceBisectExt<T>for[T]{fn find_bisect(&self,f:impl FnMut(&T)->bool)->Option<&T>{self.get(self.position_bisect(f))}fn rfind_bisect(&self,f:impl FnMut(&T)->bool)->Option<&T>{let pos=self.rposition_bisect(f);if pos==0{None}else{self.get(pos-1)}}fn position_bisect(&self,mut f:impl FnMut(&T)->bool)->usize{binary_search(|i|f(&self[*i as usize]),self.len()as i64,-1)as usize}fn rposition_bisect(&self,mut f:impl FnMut(&T)->bool)->usize{binary_search(|i|f(&self[i-1]),0,self.len()+1)}}pub fn parallel_binary_search<T,F,G>(mut f:F,q:usize,ok:T,err:T)->Vec<T>where T:Bisect,F:FnMut(&[Option<T>])->G,G:Fn(usize)->bool{let mut ok=vec![ok;q];let mut err=vec![err;q];loop{let m:Vec<_>=ok.iter().zip(&err).map(|(ok,err)|ok.bisect_middle_point(err)).collect();if m.iter().all(|m|m.is_none()){break;}let g=f(&m);for(i,m)in m.into_iter().enumerate(){if let Some(m)=m{if g(i){ok[i]=m;}else{err[i]=m;}}}}ok}}
// codesnip-guard: bounded
#[cfg_attr(any(),rustfmt::skip)]pub use self::bounded::Bounded;#[cfg_attr(any(),rustfmt::skip)]mod bounded{#[doc=" Trait for max/min bounds"]pub trait Bounded:Sized+PartialOrd{fn maximum()->Self;fn minimum()->Self;fn is_maximum(&self)->bool{self==&Self::maximum()}fn is_minimum(&self)->bool{self==&Self::minimum()}fn set_maximum(&mut self){*self=Self::maximum()}fn set_minimum(&mut self){*self=Self::minimum()}}macro_rules!impl_bounded_num{($($t:ident)*)=>{$(impl Bounded for$t{fn maximum()->Self{$t::MAX}fn minimum()->Self{$t::MIN}})*};}impl_bounded_num!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64);macro_rules!impl_bounded_tuple{(@impl$($T:ident)*)=>{impl<$($T:Bounded),*>Bounded for($($T,)*){fn maximum()->Self{($(<$T as Bounded>::maximum(),)*)}fn minimum()->Self{($(<$T as Bounded>::minimum(),)*)}}};(@inner$($T:ident)*,)=>{impl_bounded_tuple!(@impl$($T)*);};(@inner$($T:ident)*,$U:ident$($Rest:ident)*)=>{impl_bounded_tuple!(@impl$($T)*);impl_bounded_tuple!(@inner$($T)*$U,$($Rest)*);};($T:ident$($Rest:ident)*)=>{impl_bounded_tuple!(@inner$T,$($Rest)*);};}impl_bounded_tuple!(A B C D E F G H I J);impl Bounded for(){fn maximum()->Self{}fn minimum()->Self{}}impl Bounded for bool{fn maximum()->Self{true}fn minimum()->Self{false}}impl<T>Bounded for Option<T>where T:Bounded{fn maximum()->Self{Some(<T as Bounded>::maximum())}fn minimum()->Self{None}}impl<T>Bounded for std::cmp::Reverse<T>where T:Bounded{fn maximum()->Self{std::cmp::Reverse(<T as Bounded>::minimum())}fn minimum()->Self{std::cmp::Reverse(<T as Bounded>::maximum())}}}
// codesnip-guard: compress
#[cfg_attr(any(),rustfmt::skip)]pub use self::compress::{Compressor,HashCompress,VecCompress};#[cfg_attr(any(),rustfmt::skip)]mod compress{use super::SliceBisectExt;use std::{collections::HashMap,fmt::{self,Debug},hash::Hash,iter::FromIterator};pub trait Compressor<T>where Self:FromIterator<T>,T:Ord{fn index_exact(&self,index:&T)->Option<usize>;fn index_lower_bound(&self,index:&T)->usize;fn size(&self)->usize;}#[derive(Debug,Clone)]pub struct VecCompress<T>{data:Vec<T>}impl<T>VecCompress<T>{pub fn from_sorted_unique(data:Vec<T>)->Self{Self{data}}pub fn values(&self)->&[T]{&self.data}}impl<T>FromIterator<T>for VecCompress<T>where T:Ord{fn from_iter<I>(iter:I)->Self where I:IntoIterator<Item=T>{let mut data:Vec<_>=iter.into_iter().collect();data.sort_unstable();data.dedup();Self{data}}}impl<T>Compressor<T>for VecCompress<T>where T:Ord{fn index_exact(&self,index:&T)->Option<usize>{self.data.binary_search(index).ok()}fn index_lower_bound(&self,index:&T)->usize{self.data.position_bisect(|x|x>=index)}fn size(&self)->usize{self.data.len()}}#[derive(Clone)]pub struct HashCompress<T>{data:HashMap<T,usize>}impl<T>Debug for HashCompress<T>where T:Debug+Eq+Hash{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{f.debug_struct("HashCompress").field("data",&self.data).finish()}}impl<T>FromIterator<T>for HashCompress<T>where T:Ord+Hash{fn from_iter<I>(iter:I)->Self where I:IntoIterator<Item=T>{let mut data:Vec<_>=iter.into_iter().collect();data.sort_unstable();data.dedup();let data=data.into_iter().enumerate().map(|(i,t)|(t,i)).collect();Self{data}}}impl<T>Compressor<T>for HashCompress<T>where T:Ord+Hash{fn index_exact(&self,index:&T)->Option<usize>{self.data.get(index).copied()}fn index_lower_bound(&self,_index:&T)->usize{panic!("HashCompress does not implement index_lower_bound")}fn size(&self)->usize{self.data.len()}}}
// codesnip-guard: discrete_steps
#[cfg_attr(any(),rustfmt::skip)]pub use self::discrete_steps::{DiscreteSteps,RangeBoundsExt};#[cfg_attr(any(),rustfmt::skip)]mod discrete_steps{use super::Bounded;use std::{convert::TryFrom,ops::{Bound,Range,RangeBounds,RangeInclusive}};pub trait DiscreteSteps<Delta>:Clone{fn delta()->Delta;fn steps_between(start:&Self,end:&Self)->Option<Delta>;fn forward_checked(start:Self,delta:Delta)->Option<Self>;fn backward_checked(start:Self,delta:Delta)->Option<Self>;fn forward(start:Self,delta:Delta)->Self{Self::forward_checked(start,delta).expect("overflow in `DiscreteSteps::forward`")}fn backward(start:Self,delta:Delta)->Self{Self::backward_checked(start,delta).expect("overflow in `DiscreteSteps::backward`")}fn forward_delta_checked(start:Self)->Option<Self>{Self::forward_checked(start,Self::delta())}fn backward_delta_checked(start:Self)->Option<Self>{Self::backward_checked(start,Self::delta())}fn forward_delta(start:Self)->Self{Self::forward(start,Self::delta())}fn backward_delta(start:Self)->Self{Self::backward(start,Self::delta())}}macro_rules!impl_discrete_steps_integer{(@common$u_source:ident)=>{fn delta()->$u_source{1}fn forward(start:Self,delta:$u_source)->Self{assert!(Self::forward_checked(start,delta).is_some(),"attempt to add with overflow");start.wrapping_add(delta as Self)}fn backward(start:Self,delta:$u_source)->Self{assert!(Self::backward_checked(start,delta).is_some(),"attempt to subtract with overflow");start.wrapping_sub(delta as Self)}};($u_source:ident$i_source:ident;$($u_narrower:ident$i_narrower:ident),*;$($u_wider:ident$i_wider:ident),*)=>{$(impl DiscreteSteps<$u_source>for$u_narrower{impl_discrete_steps_integer!(@common$u_source);fn steps_between(start:&Self,end:&Self)->Option<$u_source>{if*start<=*end{Some((*end-*start)as$u_source)}else{None}}fn forward_checked(start:Self,delta:$u_source)->Option<Self>{Self::try_from(delta).ok().and_then(|delta|start.checked_add(delta))}fn backward_checked(start:Self,delta:$u_source)->Option<Self>{Self::try_from(delta).ok().and_then(|delta|start.checked_sub(delta))}}impl DiscreteSteps<$u_source>for$i_narrower{impl_discrete_steps_integer!(@common$u_source);fn steps_between(start:&Self,end:&Self)->Option<$u_source>{if*start<=*end{Some((*end as$i_source).wrapping_sub(*start as$i_source)as$u_source)}else{None}}fn forward_checked(start:Self,delta:$u_source)->Option<Self>{$u_narrower::try_from(delta).ok().and_then(|delta|{let wrapped=start.wrapping_add(delta as Self);if wrapped>=start{Some(wrapped)}else{None}})}fn backward_checked(start:Self,delta:$u_source)->Option<Self>{$u_narrower::try_from(delta).ok().and_then(|delta|{let wrapped=start.wrapping_sub(delta as Self);if wrapped<=start{Some(wrapped)}else{None}})}})*$(impl DiscreteSteps<$u_source>for$u_wider{impl_discrete_steps_integer!(@common$u_source);fn steps_between(start:&Self,end:&Self)->Option<$u_source>{if*start<=*end{$u_source::try_from(*end-*start).ok()}else{None}}fn forward_checked(start:Self,delta:$u_source)->Option<Self>{start.checked_add(delta as Self)}fn backward_checked(start:Self,delta:$u_source)->Option<Self>{start.checked_sub(delta as Self)}}impl DiscreteSteps<$u_source>for$i_wider{impl_discrete_steps_integer!(@common$u_source);fn steps_between(start:&Self,end:&Self)->Option<$u_source>{if*start<=*end{end.checked_sub(*start).and_then(|result|$u_source::try_from(result).ok())}else{None}}fn forward_checked(start:Self,delta:$u_source)->Option<Self>{start.checked_add(delta as Self)}fn backward_checked(start:Self,delta:$u_source)->Option<Self>{start.checked_sub(delta as Self)}})*};}impl_discrete_steps_integer!(u16 i16;u8 i8,u16 i16,usize isize;u32 i32,u64 i64,u128 i128);impl_discrete_steps_integer!(u32 i32;u8 i8,u16 i16,u32 i32,usize isize;u64 i64,u128 i128);impl_discrete_steps_integer!(u64 i64;u8 i8,u16 i16,u32 i32,u64 i64,usize isize;u128 i128);impl_discrete_steps_integer!(u128 i128;u8 i8,u16 i16,u32 i32,u64 i64,u128 i128,usize isize;);impl_discrete_steps_integer!(usize isize;u8 i8,u16 i16,u32 i32,u64 i64,usize isize;u128 i128);pub trait RangeBoundsExt<T>{fn start_bound_included_checked(&self)->Option<T>;fn start_bound_excluded_checked(&self)->Option<T>;fn end_bound_included_checked(&self)->Option<T>;fn end_bound_excluded_checked(&self)->Option<T>;fn start_bound_included(&self)->T;fn start_bound_excluded(&self)->T;fn end_bound_included(&self)->T;fn end_bound_excluded(&self)->T;fn start_bound_included_bounded(&self,lb:T)->Option<T>where T:Ord;fn start_bound_excluded_bounded(&self,lb:T)->Option<T>where T:Ord;fn end_bound_included_bounded(&self,ub:T)->Option<T>where T:Ord;fn end_bound_excluded_bounded(&self,ub:T)->Option<T>where T:Ord;fn to_range_checked(&self)->Option<Range<T>>{match(self.start_bound_included_checked(),self.end_bound_excluded_checked()){(Some(start),Some(end))=>Some(start..end),_=>None,}}fn to_range(&self)->Range<T>{self.start_bound_included()..self.end_bound_excluded()}fn to_range_bounded(&self,min:T,max:T)->Option<Range<T>>where T:Ord{Some(self.start_bound_included_bounded(min)?..self.end_bound_excluded_bounded(max)?)}fn to_range_inclusive_checked(&self)->Option<RangeInclusive<T>>{match(self.start_bound_included_checked(),self.end_bound_included_checked()){(Some(start),Some(end))=>Some(start..=end),_=>None,}}fn to_range_inclusive(&self)->RangeInclusive<T>{self.start_bound_included()..=self.end_bound_included()}fn to_range_inclusive_bounded(&self,min:T,max:T)->Option<RangeInclusive<T>>where T:Ord{Some(self.start_bound_included_bounded(min)?..=self.end_bound_included_bounded(max)?)}}macro_rules!impl_range_bounds_ext{($($source:ident=>$($target:ident)+);*$(;)?)=>{$($(impl<R>RangeBoundsExt<$target>for R where R:RangeBounds<$target>,{fn start_bound_included_checked(&self)->Option<$target>{match self.start_bound(){Bound::Included(x)=>Some(*x),Bound::Excluded(x)=>DiscreteSteps::<$source>::forward_delta_checked(*x),Bound::Unbounded=>Some(Bounded::minimum()),}}fn start_bound_excluded_checked(&self)->Option<$target>{match self.start_bound(){Bound::Included(x)=>DiscreteSteps::<$source>::backward_delta_checked(*x),Bound::Excluded(x)=>Some(*x),Bound::Unbounded=>None,}}fn end_bound_included_checked(&self)->Option<$target>{match self.end_bound(){Bound::Included(x)=>Some(*x),Bound::Excluded(x)=>DiscreteSteps::<$source>::backward_delta_checked(*x),Bound::Unbounded=>Some(Bounded::maximum()),}}fn end_bound_excluded_checked(&self)->Option<$target>{match self.end_bound(){Bound::Included(x)=>DiscreteSteps::<$source>::forward_delta_checked(*x),Bound::Excluded(x)=>Some(*x),Bound::Unbounded=>None,}}fn start_bound_included(&self)->$target{match self.start_bound(){Bound::Included(x)=>*x,Bound::Excluded(x)=>DiscreteSteps::<$source>::forward_delta(*x),Bound::Unbounded=>Bounded::minimum(),}}fn start_bound_excluded(&self)->$target{match self.start_bound(){Bound::Included(x)=>DiscreteSteps::<$source>::backward_delta(*x),Bound::Excluded(x)=>*x,Bound::Unbounded=>DiscreteSteps::<$source>::backward_delta(Bounded::minimum()),}}fn end_bound_included(&self)->$target{match self.end_bound(){Bound::Included(x)=>*x,Bound::Excluded(x)=>DiscreteSteps::<$source>::backward_delta(*x),Bound::Unbounded=>Bounded::maximum(),}}fn end_bound_excluded(&self)->$target{match self.end_bound(){Bound::Included(x)=>DiscreteSteps::<$source>::forward_delta(*x),Bound::Excluded(x)=>*x,Bound::Unbounded=>DiscreteSteps::<$source>::forward_delta(Bounded::maximum()),}}fn start_bound_included_bounded(&self,lb:$target)->Option<$target>where$target:Ord{match self.start_bound(){Bound::Included(x)=>Some(*x).filter(|&x|lb<=x),Bound::Excluded(x)=>DiscreteSteps::<$source>::forward_delta_checked(*x).filter(|&x|lb<=x),Bound::Unbounded=>Some(lb),}}fn start_bound_excluded_bounded(&self,lb:$target)->Option<$target>where$target:Ord{match self.start_bound(){Bound::Included(x)=>DiscreteSteps::<$source>::backward_delta_checked(*x).filter(|&x|lb<=x),Bound::Excluded(x)=>Some(*x).filter(|&x|lb<=x),Bound::Unbounded=>Some(lb),}}fn end_bound_included_bounded(&self,ub:$target)->Option<$target>where$target:Ord{match self.end_bound(){Bound::Included(x)=>Some(*x).filter(|&x|x<=ub),Bound::Excluded(x)=>DiscreteSteps::<$source>::backward_delta_checked(*x).filter(|&x|x<=ub),Bound::Unbounded=>Some(ub),}}fn end_bound_excluded_bounded(&self,ub:$target)->Option<$target>where$target:Ord{match self.end_bound(){Bound::Included(x)=>DiscreteSteps::<$source>::forward_delta_checked(*x).filter(|&x|x<=ub),Bound::Excluded(x)=>Some(*x).filter(|&x|x<=ub),Bound::Unbounded=>Some(ub),}}})+)*};}impl_range_bounds_ext!(u16=>u8 i8 u16 i16;u32=>u32 i32;u64=>u64 i64;u128=>u128 i128;usize=>isize usize;);}
// codesnip-guard: simd
#[cfg_attr(any(),rustfmt::skip)]mod simd{#![allow(unsafe_op_in_unsafe_fn)]#[cfg(target_arch="x86_64")]use std::arch::x86_64::*;#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]pub unsafe fn first_ge_u16x32_avx2(values:&[u16;32],key:u16)->usize{let sign=_mm256_set1_epi16(i16::MIN);let key=_mm256_xor_si256(_mm256_set1_epi16(key as i16),sign);let low=_mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().cast()),sign);let high=_mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().add(16).cast()),sign);let low=_mm256_movemask_epi8(_mm256_cmpgt_epi16(key,low))as u32 as u64;let high=_mm256_movemask_epi8(_mm256_cmpgt_epi16(key,high))as u32 as u64;(!(low|high<<32)).trailing_zeros()as usize/2}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]pub unsafe fn first_gt_u16x32_avx2(values:&[u16;32],key:u16)->usize{let sign=_mm256_set1_epi16(i16::MIN);let key=_mm256_xor_si256(_mm256_set1_epi16(key as i16),sign);let low=_mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().cast()),sign);let high=_mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().add(16).cast()),sign);let low=_mm256_movemask_epi8(_mm256_cmpgt_epi16(low,key))as u32 as u64;let high=_mm256_movemask_epi8(_mm256_cmpgt_epi16(high,key))as u32 as u64;(low|high<<32).trailing_zeros()as usize/2}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]pub unsafe fn first_ge_u32x16_avx2(values:&[u32;16],key:u32)->usize{let sign=_mm256_set1_epi32(i32::MIN);let key=_mm256_xor_si256(_mm256_set1_epi32(key as i32),sign);let low=_mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().cast()),sign);let high=_mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().add(8).cast()),sign);let low=_mm256_movemask_ps(_mm256_castsi256_ps(_mm256_cmpgt_epi32(key,low)));let high=_mm256_movemask_ps(_mm256_castsi256_ps(_mm256_cmpgt_epi32(key,high)));let mask=((!low as u32)&0xff)|(((!high as u32)&0xff)<<8);(mask as u16).trailing_zeros()as usize}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]pub unsafe fn first_gt_u32x16_avx2(values:&[u32;16],key:u32)->usize{let sign=_mm256_set1_epi32(i32::MIN);let key=_mm256_xor_si256(_mm256_set1_epi32(key as i32),sign);let low=_mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().cast()),sign);let high=_mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().add(8).cast()),sign);let low=_mm256_movemask_ps(_mm256_castsi256_ps(_mm256_cmpgt_epi32(low,key)));let high=_mm256_movemask_ps(_mm256_castsi256_ps(_mm256_cmpgt_epi32(high,key)));let mask=low as u32|((high as u32)<<8);(mask as u16).trailing_zeros()as usize}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]pub unsafe fn first_ge_u64x8_avx2(values:&[u64;8],key:u64)->usize{let sign=_mm256_set1_epi64x(i64::MIN);let key=_mm256_xor_si256(_mm256_set1_epi64x(key as i64),sign);let low=_mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().cast()),sign);let high=_mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().add(4).cast()),sign);let low=_mm256_movemask_pd(_mm256_castsi256_pd(_mm256_cmpgt_epi64(key,low)));let high=_mm256_movemask_pd(_mm256_castsi256_pd(_mm256_cmpgt_epi64(key,high)));let mask=((!low as u32)&0x0f)|(((!high as u32)&0x0f)<<4);(mask as u8).trailing_zeros()as usize}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]pub unsafe fn first_gt_u64x8_avx2(values:&[u64;8],key:u64)->usize{let sign=_mm256_set1_epi64x(i64::MIN);let key=_mm256_xor_si256(_mm256_set1_epi64x(key as i64),sign);let low=_mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().cast()),sign);let high=_mm256_xor_si256(_mm256_loadu_si256(values.as_ptr().add(4).cast()),sign);let low=_mm256_movemask_pd(_mm256_castsi256_pd(_mm256_cmpgt_epi64(low,key)));let high=_mm256_movemask_pd(_mm256_castsi256_pd(_mm256_cmpgt_epi64(high,key)));let mask=low as u32|((high as u32)<<4);(mask as u8).trailing_zeros()as usize}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f,avx512bw")]pub unsafe fn first_ge_u16x32_avx512(values:&[u16;32],key:u16)->usize{let values=_mm512_loadu_si512(values.as_ptr().cast());let key=_mm512_set1_epi16(key as i16);(!_mm512_cmplt_epu16_mask(values,key)).trailing_zeros()as usize}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f,avx512bw")]pub unsafe fn first_gt_u16x32_avx512(values:&[u16;32],key:u16)->usize{let values=_mm512_loadu_si512(values.as_ptr().cast());let key=_mm512_set1_epi16(key as i16);_mm512_cmpgt_epu16_mask(values,key).trailing_zeros()as usize}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f")]pub unsafe fn first_ge_u32x16_avx512(values:&[u32;16],key:u32)->usize{let values=_mm512_loadu_si512(values.as_ptr().cast());let key=_mm512_set1_epi32(key as i32);(!_mm512_cmplt_epu32_mask(values,key)).trailing_zeros()as usize}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f")]pub unsafe fn first_gt_u32x16_avx512(values:&[u32;16],key:u32)->usize{let values=_mm512_loadu_si512(values.as_ptr().cast());let key=_mm512_set1_epi32(key as i32);_mm512_cmpgt_epu32_mask(values,key).trailing_zeros()as usize}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f")]pub unsafe fn first_ge_u64x8_avx512(values:&[u64;8],key:u64)->usize{let values=_mm512_loadu_si512(values.as_ptr().cast());let key=_mm512_set1_epi64(key as i64);(!_mm512_cmplt_epu64_mask(values,key)).trailing_zeros()as usize}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f")]pub unsafe fn first_gt_u64x8_avx512(values:&[u64;8],key:u64)->usize{let values=_mm512_loadu_si512(values.as_ptr().cast());let key=_mm512_set1_epi64(key as i64);_mm512_cmpgt_epu64_mask(values,key).trailing_zeros()as usize}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]pub unsafe fn add_suffix_u32x16_avx2(values:&mut[u32;16],index:usize,delta:u32){let index=_mm256_set1_epi32(index as i32-1);let delta=_mm256_set1_epi32(delta as i32);let low_mask=_mm256_cmpgt_epi32(_mm256_setr_epi32(0,1,2,3,4,5,6,7),index);let high_mask=_mm256_cmpgt_epi32(_mm256_setr_epi32(8,9,10,11,12,13,14,15),index);let low=_mm256_add_epi32(_mm256_loadu_si256(values.as_ptr().cast()),_mm256_and_si256(delta,low_mask));let high=_mm256_add_epi32(_mm256_loadu_si256(values.as_ptr().add(8).cast()),_mm256_and_si256(delta,high_mask));_mm256_storeu_si256(values.as_mut_ptr().cast(),low);_mm256_storeu_si256(values.as_mut_ptr().add(8).cast(),high);}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]pub unsafe fn add_suffix_u64x8_avx2(values:&mut[u64;8],index:usize,delta:u64){let index=_mm256_set1_epi64x(index as i64-1);let delta=_mm256_set1_epi64x(delta as i64);let low_mask=_mm256_cmpgt_epi64(_mm256_setr_epi64x(0,1,2,3),index);let high_mask=_mm256_cmpgt_epi64(_mm256_setr_epi64x(4,5,6,7),index);let low=_mm256_add_epi64(_mm256_loadu_si256(values.as_ptr().cast()),_mm256_and_si256(delta,low_mask));let high=_mm256_add_epi64(_mm256_loadu_si256(values.as_ptr().add(4).cast()),_mm256_and_si256(delta,high_mask));_mm256_storeu_si256(values.as_mut_ptr().cast(),low);_mm256_storeu_si256(values.as_mut_ptr().add(4).cast(),high);}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f")]pub unsafe fn add_suffix_u32x16_avx512(values:&mut[u32;16],index:usize,delta:u32){let values_vector=_mm512_loadu_si512(values.as_ptr().cast());let values_vector=_mm512_mask_add_epi32(values_vector,u16::MAX<<index,values_vector,_mm512_set1_epi32(delta as i32));_mm512_storeu_si512(values.as_mut_ptr().cast(),values_vector);}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f")]pub unsafe fn add_suffix_u64x8_avx512(values:&mut[u64;8],index:usize,delta:u64){let values_vector=_mm512_loadu_si512(values.as_ptr().cast());let values_vector=_mm512_mask_add_epi64(values_vector,u8::MAX<<index,values_vector,_mm512_set1_epi64(delta as i64));_mm512_storeu_si512(values.as_mut_ptr().cast(),values_vector);}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]pub unsafe fn max_index_u32x16_avx2(values:&[u32;16])->usize{let low=_mm256_loadu_si256(values.as_ptr().cast());let high=_mm256_loadu_si256(values.as_ptr().add(8).cast());let mut maximum=_mm256_max_epu32(low,high);maximum=_mm256_max_epu32(maximum,_mm256_permute2x128_si256::<0x01>(maximum,maximum));maximum=_mm256_max_epu32(maximum,_mm256_shuffle_epi32::<0x4e>(maximum));maximum=_mm256_max_epu32(maximum,_mm256_shuffle_epi32::<0xb1>(maximum));let maximum=_mm256_set1_epi32(_mm256_extract_epi32::<0>(maximum));let low=_mm256_movemask_ps(_mm256_castsi256_ps(_mm256_cmpeq_epi32(low,maximum)));let high=_mm256_movemask_ps(_mm256_castsi256_ps(_mm256_cmpeq_epi32(high,maximum)));((low as u32|((high as u32)<<8))as u16).trailing_zeros()as usize}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f")]pub unsafe fn max_index_u32x16_avx512(values:&[u32;16])->usize{let values=_mm512_loadu_si512(values.as_ptr().cast());let maximum=_mm512_set1_epi32(_mm512_reduce_max_epu32(values)as i32);_mm512_cmpeq_epi32_mask(values,maximum).trailing_zeros()as usize}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]unsafe fn max_epu64(left:__m256i,right:__m256i)->__m256i{let sign=_mm256_set1_epi64x(i64::MIN);let greater=_mm256_cmpgt_epi64(_mm256_xor_si256(left,sign),_mm256_xor_si256(right,sign));_mm256_blendv_epi8(right,left,greater)}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]pub unsafe fn max_index_u64x8_avx2(values:&[u64;8])->usize{let low=_mm256_loadu_si256(values.as_ptr().cast());let high=_mm256_loadu_si256(values.as_ptr().add(4).cast());let mut maximum=max_epu64(low,high);maximum=max_epu64(maximum,_mm256_permute4x64_epi64::<0x4e>(maximum));maximum=max_epu64(maximum,_mm256_permute4x64_epi64::<0xb1>(maximum));let maximum=_mm256_set1_epi64x(_mm256_extract_epi64::<0>(maximum));let low=_mm256_movemask_pd(_mm256_castsi256_pd(_mm256_cmpeq_epi64(low,maximum)));let high=_mm256_movemask_pd(_mm256_castsi256_pd(_mm256_cmpeq_epi64(high,maximum)));((low as u32|((high as u32)<<4))as u8).trailing_zeros()as usize}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f")]pub unsafe fn max_index_u64x8_avx512(values:&[u64;8])->usize{let values=_mm512_loadu_si512(values.as_ptr().cast());let maximum=_mm512_set1_epi64(_mm512_reduce_max_epu64(values)as i64);_mm512_cmpeq_epi64_mask(values,maximum).trailing_zeros()as usize}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]pub unsafe fn max_index_u128x4_avx2(low:&[u64;4],high:&[u64;4])->usize{let low=_mm256_loadu_si256(low.as_ptr().cast());let high=_mm256_loadu_si256(high.as_ptr().cast());let mut maximum_high=high;maximum_high=max_epu64(maximum_high,_mm256_permute4x64_epi64::<0x4e>(maximum_high));maximum_high=max_epu64(maximum_high,_mm256_permute4x64_epi64::<0xb1>(maximum_high));let high_equal=_mm256_cmpeq_epi64(high,maximum_high);let high_mask=_mm256_movemask_pd(_mm256_castsi256_pd(high_equal))as u32&15;if high_mask.is_power_of_two(){return high_mask.trailing_zeros()as usize;}let mut maximum_low=_mm256_and_si256(low,high_equal);maximum_low=max_epu64(maximum_low,_mm256_permute4x64_epi64::<0x4e>(maximum_low));maximum_low=max_epu64(maximum_low,_mm256_permute4x64_epi64::<0xb1>(maximum_low));let both=_mm256_and_si256(high_equal,_mm256_cmpeq_epi64(low,maximum_low));(_mm256_movemask_pd(_mm256_castsi256_pd(both))as u32&15).trailing_zeros()as usize}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2,avx512f,avx512vl")]pub unsafe fn max_index_u128x4_avx512(low:&[u64;4],high:&[u64;4])->usize{let low=_mm256_loadu_si256(low.as_ptr().cast());let high=_mm256_loadu_si256(high.as_ptr().cast());let mut maximum_high=high;maximum_high=_mm256_max_epu64(maximum_high,_mm256_permute4x64_epi64::<0x4e>(maximum_high));maximum_high=_mm256_max_epu64(maximum_high,_mm256_permute4x64_epi64::<0xb1>(maximum_high));let high_equal=_mm256_cmpeq_epi64(high,maximum_high);let high_mask=_mm256_movemask_pd(_mm256_castsi256_pd(high_equal))as u32&15;if high_mask.is_power_of_two(){return high_mask.trailing_zeros()as usize;}let mut maximum_low=_mm256_and_si256(low,high_equal);maximum_low=_mm256_max_epu64(maximum_low,_mm256_permute4x64_epi64::<0x4e>(maximum_low));maximum_low=_mm256_max_epu64(maximum_low,_mm256_permute4x64_epi64::<0xb1>(maximum_low));let both=_mm256_and_si256(high_equal,_mm256_cmpeq_epi64(low,maximum_low));(_mm256_movemask_pd(_mm256_castsi256_pd(both))as u32&15).trailing_zeros()as usize}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]unsafe fn reduce_min_i32x8(mut values:__m256i)->i32{values=_mm256_min_epi32(values,_mm256_permute2x128_si256::<0x01>(values,values));values=_mm256_min_epi32(values,_mm256_shuffle_epi32::<0x4e>(values));values=_mm256_min_epi32(values,_mm256_shuffle_epi32::<0xb1>(values));_mm256_extract_epi32::<0>(values)}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]unsafe fn reduce_max_i32x8(mut values:__m256i)->i32{values=_mm256_max_epi32(values,_mm256_permute2x128_si256::<0x01>(values,values));values=_mm256_max_epi32(values,_mm256_shuffle_epi32::<0x4e>(values));values=_mm256_max_epi32(values,_mm256_shuffle_epi32::<0xb1>(values));_mm256_extract_epi32::<0>(values)}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]pub unsafe fn minimum_i32x16_avx2(values:&[i32;16])->i32{let low=_mm256_loadu_si256(values.as_ptr().cast());let high=_mm256_loadu_si256(values.as_ptr().add(8).cast());reduce_min_i32x8(_mm256_min_epi32(low,high))}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]pub unsafe fn maximum_i32x16_avx2(values:&[i32;16])->i32{let low=_mm256_loadu_si256(values.as_ptr().cast());let high=_mm256_loadu_si256(values.as_ptr().add(8).cast());reduce_max_i32x8(_mm256_max_epi32(low,high))}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]unsafe fn range_mask_i32x8(start:usize,end:usize,offset:i32)->__m256i{let lanes=_mm256_setr_epi32(offset,offset+1,offset+2,offset+3,offset+4,offset+5,offset+6,offset+7);_mm256_and_si256(_mm256_cmpgt_epi32(lanes,_mm256_set1_epi32(start as i32-1)),_mm256_cmpgt_epi32(_mm256_set1_epi32(end as i32),lanes))}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]pub unsafe fn minimum_range_i32x16_avx2(values:&[i32;16],start:usize,end:usize)->i32{let unit=_mm256_set1_epi32(i32::MAX);let low=_mm256_blendv_epi8(unit,_mm256_loadu_si256(values.as_ptr().cast()),range_mask_i32x8(start,end,0));let high=_mm256_blendv_epi8(unit,_mm256_loadu_si256(values.as_ptr().add(8).cast()),range_mask_i32x8(start,end,8));reduce_min_i32x8(_mm256_min_epi32(low,high))}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]pub unsafe fn maximum_range_i32x16_avx2(values:&[i32;16],start:usize,end:usize)->i32{let unit=_mm256_set1_epi32(i32::MIN);let low=_mm256_blendv_epi8(unit,_mm256_loadu_si256(values.as_ptr().cast()),range_mask_i32x8(start,end,0));let high=_mm256_blendv_epi8(unit,_mm256_loadu_si256(values.as_ptr().add(8).cast()),range_mask_i32x8(start,end,8));reduce_max_i32x8(_mm256_max_epi32(low,high))}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f")]pub unsafe fn minimum_i32x16_avx512(values:&[i32;16])->i32{_mm512_reduce_min_epi32(_mm512_loadu_si512(values.as_ptr().cast()))}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f")]pub unsafe fn maximum_i32x16_avx512(values:&[i32;16])->i32{_mm512_reduce_max_epi32(_mm512_loadu_si512(values.as_ptr().cast()))}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f")]pub unsafe fn minimum_range_i32x16_avx512(values:&[i32;16],start:usize,end:usize)->i32{let mask=(u16::MAX<<start)&(u16::MAX>>(16-end));_mm512_mask_reduce_min_epi32(mask,_mm512_loadu_si512(values.as_ptr().cast()))}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f")]pub unsafe fn maximum_range_i32x16_avx512(values:&[i32;16],start:usize,end:usize)->i32{let mask=(u16::MAX<<start)&(u16::MAX>>(16-end));_mm512_mask_reduce_max_epi32(mask,_mm512_loadu_si512(values.as_ptr().cast()))}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]unsafe fn min_i64x4(left:__m256i,right:__m256i)->__m256i{_mm256_blendv_epi8(left,right,_mm256_cmpgt_epi64(left,right))}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]unsafe fn max_i64x4(left:__m256i,right:__m256i)->__m256i{_mm256_blendv_epi8(right,left,_mm256_cmpgt_epi64(left,right))}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]unsafe fn reduce_min_i64x4(mut values:__m256i)->i64{values=min_i64x4(values,_mm256_permute4x64_epi64::<0x4e>(values));values=min_i64x4(values,_mm256_permute4x64_epi64::<0xb1>(values));_mm256_extract_epi64::<0>(values)}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]unsafe fn reduce_max_i64x4(mut values:__m256i)->i64{values=max_i64x4(values,_mm256_permute4x64_epi64::<0x4e>(values));values=max_i64x4(values,_mm256_permute4x64_epi64::<0xb1>(values));_mm256_extract_epi64::<0>(values)}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]pub unsafe fn minimum_i64x8_avx2(values:&[i64;8])->i64{let low=_mm256_loadu_si256(values.as_ptr().cast());let high=_mm256_loadu_si256(values.as_ptr().add(4).cast());reduce_min_i64x4(min_i64x4(low,high))}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]pub unsafe fn maximum_i64x8_avx2(values:&[i64;8])->i64{let low=_mm256_loadu_si256(values.as_ptr().cast());let high=_mm256_loadu_si256(values.as_ptr().add(4).cast());reduce_max_i64x4(max_i64x4(low,high))}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]unsafe fn range_mask_i64x4(start:usize,end:usize,offset:i64)->__m256i{let lanes=_mm256_setr_epi64x(offset,offset+1,offset+2,offset+3);_mm256_and_si256(_mm256_cmpgt_epi64(lanes,_mm256_set1_epi64x(start as i64-1)),_mm256_cmpgt_epi64(_mm256_set1_epi64x(end as i64),lanes))}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]pub unsafe fn minimum_range_i64x8_avx2(values:&[i64;8],start:usize,end:usize)->i64{let unit=_mm256_set1_epi64x(i64::MAX);let low=_mm256_blendv_epi8(unit,_mm256_loadu_si256(values.as_ptr().cast()),range_mask_i64x4(start,end,0));let high=_mm256_blendv_epi8(unit,_mm256_loadu_si256(values.as_ptr().add(4).cast()),range_mask_i64x4(start,end,4));reduce_min_i64x4(min_i64x4(low,high))}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx2")]pub unsafe fn maximum_range_i64x8_avx2(values:&[i64;8],start:usize,end:usize)->i64{let unit=_mm256_set1_epi64x(i64::MIN);let low=_mm256_blendv_epi8(unit,_mm256_loadu_si256(values.as_ptr().cast()),range_mask_i64x4(start,end,0));let high=_mm256_blendv_epi8(unit,_mm256_loadu_si256(values.as_ptr().add(4).cast()),range_mask_i64x4(start,end,4));reduce_max_i64x4(max_i64x4(low,high))}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f")]pub unsafe fn minimum_i64x8_avx512(values:&[i64;8])->i64{_mm512_reduce_min_epi64(_mm512_loadu_si512(values.as_ptr().cast()))}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f")]pub unsafe fn maximum_i64x8_avx512(values:&[i64;8])->i64{_mm512_reduce_max_epi64(_mm512_loadu_si512(values.as_ptr().cast()))}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f")]pub unsafe fn minimum_range_i64x8_avx512(values:&[i64;8],start:usize,end:usize)->i64{let mask=(u8::MAX<<start)&(u8::MAX>>(8-end));_mm512_mask_reduce_min_epi64(mask,_mm512_loadu_si512(values.as_ptr().cast()))}#[cfg(target_arch="x86_64")]#[target_feature(enable="avx512f")]pub unsafe fn maximum_range_i64x8_avx512(values:&[i64;8],start:usize,end:usize)->i64{let mask=(u8::MAX<<start)&(u8::MAX>>(8-end));_mm512_mask_reduce_max_epi64(mask,_mm512_loadu_si512(values.as_ptr().cast()))}}
// codesnip-guard: zero_one
#[cfg_attr(any(),rustfmt::skip)]pub use self::zero_one::{One,Zero};#[cfg_attr(any(),rustfmt::skip)]mod zero_one{pub trait Zero:Sized{fn zero()->Self;#[inline]fn is_zero(&self)->bool where Self:PartialEq{self==&Self::zero()}#[inline]fn set_zero(&mut self){*self=Self::zero();}}pub trait One:Sized{fn one()->Self;#[inline]fn is_one(&self)->bool where Self:PartialEq{self==&Self::one()}#[inline]fn set_one(&mut self){*self=Self::one();}}macro_rules!impl_zero_one{($({$(<$T:ident:$Bound:ident>)?$Trait:ident$method:ident$($t:ty)*,$e:expr})*)=>{$(impl_zero_one!(@impl[$(<$T:$Bound>)?]$Trait$method[$($t)*],$e);)*};(@impl[<$T:ident:$Bound:ident>]$Trait:ident$method:ident[$($t:ty)*],$e:expr)=>{$(impl<$T:$Bound>$Trait for$t{fn$method()->Self{$e}})*};(@impl[]$Trait:ident$method:ident[$($t:ty)*],$e:expr)=>{$(impl$Trait for$t{fn$method()->Self{$e}})*};}impl_zero_one!({Zero zero u8 u16 u32 u64 usize i8 i16 i32 i64 isize u128 i128,0}{Zero zero f32 f64,0.}{One one u8 u16 u32 u64 usize i8 i16 i32 i64 isize u128 i128,1}{One one f32 f64,1.}{<T:Zero>Zero zero std::num::Wrapping<T>,Self(T::zero())}{<T:One>One one std::num::Wrapping<T>,Self(T::one())});}
#[allow(dead_code)]
mod bench_legacy_wavelet {
    use super::{
        AbelianGroup, BenchLegacyBitVector as BitVector, BinaryIndexedTree, Compressor, VecCompress,
    };
    use std::{
        mem::{self, MaybeUninit},
        ops::Range,
    };

    #[derive(Debug, Clone)]
    pub struct WaveletMatrix<T> {
        len: usize,
        bit_length: usize,
        zeros: Vec<usize>,
        bit_vectors: Vec<BitVector>,
        compress: VecCompress<T>,
    }

    impl<T> WaveletMatrix<T>
    where
        T: Ord + Clone,
    {
        pub fn new(v: Vec<T>) -> Self {
            let len = v.len();
            let mut sorted: Vec<_> = v
                .into_iter()
                .enumerate()
                .map(|(i, value)| (value, i))
                .collect();
            sorted.sort_unstable_by(|a, b| a.0.cmp(&b.0));
            let mut values = Vec::with_capacity(len);
            let mut indices = vec![0; len];
            for (value, i) in sorted {
                if values.last().is_none_or(|last| last != &value) {
                    values.push(value);
                }
                indices[i] = values.len() - 1;
            }
            let compress = VecCompress::from_sorted_unique(values);
            let bit_length = usize::BITS as usize - compress.size().leading_zeros() as usize;
            let mut bit_vectors = Vec::with_capacity(bit_length);
            let mut zeros = Vec::with_capacity(bit_length);
            let mut next = Vec::with_capacity(len);
            let mut ones = Vec::with_capacity(len);
            for d in (0..bit_length).rev() {
                bit_vectors.push(indices.iter().map(|&idx| ((idx >> d) & 1) != 0).collect());
                for &idx in &indices {
                    if ((idx >> d) & 1) == 0 {
                        next.push(idx);
                    } else {
                        ones.push(idx);
                    }
                }
                zeros.push(next.len());
                next.append(&mut ones);
                mem::swap(&mut indices, &mut next);
                next.clear();
            }
            Self {
                len,
                bit_length,
                zeros,
                bit_vectors,
                compress,
            }
        }

        pub fn new_with_init<F>(v: Vec<T>, mut f: F) -> Self
        where
            F: FnMut(usize, usize, T),
        {
            let this = Self::new(v.clone());
            let indices: Vec<usize> = v
                .iter()
                .map(|value| this.compress.index_exact(value).unwrap())
                .collect();
            for (mut k, value) in v.into_iter().enumerate() {
                let idx = indices[k];
                for d in (0..this.bit_length).rev() {
                    let level = this.level(d);
                    if ((idx >> d) & 1) != 0 {
                        k = this.zeros[level] + this.rank1(level, k);
                    } else {
                        k = this.rank0(level, k);
                    }
                    f(d, k, value.clone());
                }
            }
            this
        }

        fn level(&self, d: usize) -> usize {
            self.bit_length - 1 - d
        }

        fn rank1(&self, level: usize, k: usize) -> usize {
            self.bit_vectors[level].rank1(k)
        }

        fn rank0(&self, level: usize, k: usize) -> usize {
            k - self.rank1(level, k)
        }

        fn reorder<U>(&self, level: usize, current: Vec<U>, mut visit: impl FnMut(&U)) -> Vec<U> {
            assert_eq!(current.len(), self.len);
            let mut next = Vec::with_capacity(self.len);
            next.resize_with(self.len, MaybeUninit::uninit);
            let mut zero = 0;
            let mut one = self.zeros[level];
            for (i, value) in current.into_iter().enumerate() {
                visit(&value);
                if self.bit_vectors[level].access(i) {
                    next[one].write(value);
                    one += 1;
                } else {
                    next[zero].write(value);
                    zero += 1;
                }
            }
            // SAFETY: the partition counts fill every slot once, and `MaybeUninit<U>` has `U`'s layout.
            unsafe {
                let mut next = mem::ManuallyDrop::new(next);
                Vec::from_raw_parts(next.as_mut_ptr().cast(), next.len(), next.capacity())
            }
        }

        fn rank_by_index(&self, idx: usize, mut range: Range<usize>) -> usize {
            for d in (0..self.bit_length).rev() {
                let level = self.level(d);
                if ((idx >> d) & 1) != 0 {
                    range.start = self.zeros[level] + self.rank1(level, range.start);
                    range.end = self.zeros[level] + self.rank1(level, range.end);
                } else {
                    range.start = self.rank0(level, range.start);
                    range.end = self.rank0(level, range.end);
                }
            }
            range.end - range.start
        }

        /// get k-th value
        pub fn access(&self, mut k: usize) -> T {
            let mut idx = 0;
            for d in (0..self.bit_length).rev() {
                let level = self.level(d);
                if self.bit_vectors[level].access(k) {
                    idx |= 1 << d;
                    k = self.zeros[level] + self.rank1(level, k);
                } else {
                    k = self.rank0(level, k);
                }
            }
            self.compress.values()[idx].clone()
        }

        /// the number of val in range
        pub fn rank(&self, val: T, range: Range<usize>) -> usize {
            match self.compress.index_exact(&val) {
                Some(idx) => self.rank_by_index(idx, range),
                None => 0,
            }
        }

        /// index of k-th val
        pub fn select(&self, val: T, k: usize) -> Option<usize> {
            let idx = self.compress.index_exact(&val)?;
            if self.rank_by_index(idx, 0..self.len) <= k {
                return None;
            }
            let mut i = 0;
            for d in (0..self.bit_length).rev() {
                let level = self.level(d);
                if ((idx >> d) & 1) != 0 {
                    i = self.zeros[level] + self.rank1(level, i);
                } else {
                    i = self.rank0(level, i);
                }
            }
            i += k;
            for level in (0..self.bit_length).rev() {
                if i >= self.zeros[level] {
                    i = self.bit_vectors[level]
                        .select1(i - self.zeros[level])
                        .unwrap();
                } else {
                    i = self.bit_vectors[level].select0(i).unwrap();
                }
            }
            Some(i)
        }

        /// get k-th smallest value in range
        pub fn quantile(&self, mut range: Range<usize>, mut k: usize) -> T {
            let mut idx = 0;
            for d in (0..self.bit_length).rev() {
                let level = self.level(d);
                let z = self.rank0(level, range.end) - self.rank0(level, range.start);
                if z <= k {
                    k -= z;
                    idx |= 1 << d;
                    range.start = self.zeros[level] + self.rank1(level, range.start);
                    range.end = self.zeros[level] + self.rank1(level, range.end);
                } else {
                    range.start = self.rank0(level, range.start);
                    range.end = self.rank0(level, range.end);
                }
            }
            self.compress.values()[idx].clone()
        }

        pub fn quantile_batch(
            &self,
            queries: impl IntoIterator<Item = (Range<usize>, usize)>,
        ) -> Vec<T> {
            let mut queries: Vec<_> = queries
                .into_iter()
                .map(|(range, k)| [range.start as u32, range.end as u32, k as u32, 0])
                .collect();
            for d in (0..self.bit_length).rev() {
                let level = self.level(d);
                for query in &mut queries {
                    let start = query[0] as usize;
                    let end = query[1] as usize;
                    let start1 = self.rank1(level, start);
                    let end1 = self.rank1(level, end);
                    let start0 = (start - start1) as u32;
                    let end0 = (end - end1) as u32;
                    let zeros = end0 - start0;
                    let mask = 0u32.wrapping_sub((query[2] >= zeros) as u32);
                    query[0] =
                        (start0 & !mask) | ((self.zeros[level] as u32 + start1 as u32) & mask);
                    query[1] = (end0 & !mask) | ((self.zeros[level] as u32 + end1 as u32) & mask);
                    query[2] -= zeros & mask;
                    query[3] |= (1u32 << d) & mask;
                }
            }
            queries
                .into_iter()
                .map(|query| self.compress.values()[query[3] as usize].clone())
                .collect()
        }

        /// get k-th smallest value out of range
        pub fn quantile_outer(&self, mut range: Range<usize>, mut k: usize) -> T {
            let mut idx = 0;
            let mut orange = 0..self.len;
            for d in (0..self.bit_length).rev() {
                let level = self.level(d);
                let z = self.rank0(level, orange.end) - self.rank0(level, orange.start)
                    + self.rank0(level, range.start)
                    - self.rank0(level, range.end);
                if z <= k {
                    k -= z;
                    idx |= 1 << d;
                    range.start = self.zeros[level] + self.rank1(level, range.start);
                    range.end = self.zeros[level] + self.rank1(level, range.end);
                    orange.start = self.zeros[level] + self.rank1(level, orange.start);
                    orange.end = self.zeros[level] + self.rank1(level, orange.end);
                } else {
                    range.start = self.rank0(level, range.start);
                    range.end = self.rank0(level, range.end);
                    orange.start = self.rank0(level, orange.start);
                    orange.end = self.rank0(level, orange.end);
                }
            }
            self.compress.values()[idx].clone()
        }

        /// the number of value less than val in range
        pub fn rank_lessthan(&self, val: T, mut range: Range<usize>) -> usize {
            let idx = self.compress.index_lower_bound(&val);
            let mut res = 0;
            for d in (0..self.bit_length).rev() {
                let level = self.level(d);
                if ((idx >> d) & 1) != 0 {
                    res += self.rank0(level, range.end) - self.rank0(level, range.start);
                    range.start = self.zeros[level] + self.rank1(level, range.start);
                    range.end = self.zeros[level] + self.rank1(level, range.end);
                } else {
                    range.start = self.rank0(level, range.start);
                    range.end = self.rank0(level, range.end);
                }
            }
            res
        }

        /// the number of valrange in range
        pub fn rank_range(&self, valrange: Range<T>, range: Range<usize>) -> usize {
            self.rank_lessthan(valrange.end, range.clone())
                - self.rank_lessthan(valrange.start, range)
        }

        pub fn query_less_than<F>(&self, val: T, mut range: Range<usize>, mut f: F)
        where
            F: FnMut(usize, Range<usize>),
        {
            let idx = self.compress.index_lower_bound(&val);
            for d in (0..self.bit_length).rev() {
                let level = self.level(d);
                if ((idx >> d) & 1) != 0 {
                    f(
                        d,
                        self.rank0(level, range.start)..self.rank0(level, range.end),
                    );
                    range.start = self.zeros[level] + self.rank1(level, range.start);
                    range.end = self.zeros[level] + self.rank1(level, range.end);
                } else {
                    range.start = self.rank0(level, range.start);
                    range.end = self.rank0(level, range.end);
                }
            }
        }

        pub fn build_fold<M>(&self, weights: &[M::T]) -> WaveletMatrixFold<'_, T, M>
        where
            M: AbelianGroup,
        {
            let len = self.len;
            assert_eq!(weights.len(), len);
            let mut prefix = Vec::with_capacity((self.bit_length + 1) * (len + 1));
            let mut current: Vec<M::T> = weights.to_vec();
            for level in 0..self.bit_length {
                let mut acc = M::unit();
                prefix.push(acc.clone());
                current = self.reorder(level, current, |w| {
                    acc = M::operate(&acc, w);
                    prefix.push(acc.clone());
                });
            }
            let mut acc = M::unit();
            prefix.push(acc.clone());
            for w in current.into_iter() {
                acc = M::operate(&acc, &w);
                prefix.push(acc.clone());
            }
            WaveletMatrixFold {
                wavelet_matrix: self,
                prefix,
            }
        }

        pub fn build_point_add<M>(&self, weights: &[M::T]) -> WaveletMatrixPointAdd<'_, T, M>
        where
            M: AbelianGroup,
        {
            assert_eq!(weights.len(), self.len);
            let mut current = weights.to_vec();
            let mut bits = Vec::with_capacity(self.bit_length);
            for level in 0..self.bit_length {
                current = self.reorder(level, current, |_| {});
                bits.push(BinaryIndexedTree::from_slice(&current));
            }
            WaveletMatrixPointAdd {
                wavelet_matrix: self,
                bits,
            }
        }
    }

    pub struct WaveletMatrixPointAdd<'a, T, M>
    where
        T: Ord + Clone,
        M: AbelianGroup,
    {
        wavelet_matrix: &'a WaveletMatrix<T>,
        bits: Vec<BinaryIndexedTree<M>>,
    }

    impl<'a, T, M> WaveletMatrixPointAdd<'a, T, M>
    where
        T: Ord + Clone,
        M: AbelianGroup,
    {
        pub fn update(&mut self, mut index: usize, value: M::T) {
            debug_assert!(index < self.wavelet_matrix.len);
            for d in (0..self.wavelet_matrix.bit_length).rev() {
                let level = self.wavelet_matrix.level(d);
                if self.wavelet_matrix.bit_vectors[level].access(index) {
                    index =
                        self.wavelet_matrix.zeros[level] + self.wavelet_matrix.rank1(level, index);
                } else {
                    index = self.wavelet_matrix.rank0(level, index);
                }
                self.bits[level].update(index, value.clone());
            }
        }

        pub fn fold_lessthan(&self, value: T, range: Range<usize>) -> M::T {
            let mut result = M::unit();
            self.wavelet_matrix
                .query_less_than(value, range, |d, range| {
                    M::operate_assign(
                        &mut result,
                        &self.bits[self.wavelet_matrix.level(d)].fold(range.start, range.end),
                    );
                });
            result
        }

        pub fn fold_range(&self, values: Range<T>, range: Range<usize>) -> M::T {
            M::rinv_operate(
                &self.fold_lessthan(values.end, range.clone()),
                &self.fold_lessthan(values.start, range),
            )
        }
    }

    #[derive(Debug, Clone)]
    pub struct WaveletMatrixFold<'a, T, M>
    where
        T: Ord + Clone,
        M: AbelianGroup,
    {
        wavelet_matrix: &'a WaveletMatrix<T>,
        prefix: Vec<M::T>,
    }

    impl<'a, T, M> WaveletMatrixFold<'a, T, M>
    where
        T: Ord + Clone,
        M: AbelianGroup,
    {
        #[inline]
        fn range_sum(&self, level: usize, range: Range<usize>) -> M::T {
            let offset = level * (self.wavelet_matrix.len + 1);
            unsafe {
                M::rinv_operate(
                    self.prefix.get_unchecked(offset + range.end),
                    self.prefix.get_unchecked(offset + range.start),
                )
            }
        }

        pub fn fold_lessthan(&self, val: T, range: Range<usize>) -> M::T {
            self.fold_lessthan_with_count(val, range).1
        }

        pub fn fold_lessthan_with_count(&self, val: T, mut range: Range<usize>) -> (usize, M::T) {
            debug_assert!(range.end <= self.wavelet_matrix.len);
            let idx = self.wavelet_matrix.compress.index_lower_bound(&val);
            let mut count = 0;
            let mut sum = M::unit();
            for d in (0..self.wavelet_matrix.bit_length).rev() {
                let level = self.wavelet_matrix.level(d);
                let start0 = self.wavelet_matrix.rank0(level, range.start);
                let end0 = self.wavelet_matrix.rank0(level, range.end);
                if ((idx >> d) & 1) != 0 {
                    count += end0 - start0;
                    sum = M::operate(&sum, &self.range_sum(level + 1, start0..end0));
                    range.start = self.wavelet_matrix.zeros[level] + (range.start - start0);
                    range.end = self.wavelet_matrix.zeros[level] + (range.end - end0);
                } else {
                    range.start = start0;
                    range.end = end0;
                }
            }
            (count, sum)
        }

        pub fn fold_range(&self, valrange: Range<T>, range: Range<usize>) -> M::T {
            M::rinv_operate(
                &self.fold_lessthan(valrange.end, range.clone()),
                &self.fold_lessthan(valrange.start, range),
            )
        }

        pub fn fold_range_with_count(
            &self,
            valrange: Range<T>,
            range: Range<usize>,
        ) -> (usize, M::T) {
            let (count_upper, sum_upper) =
                self.fold_lessthan_with_count(valrange.end, range.clone());
            let (count_lower, sum_lower) = self.fold_lessthan_with_count(valrange.start, range);
            (
                count_upper - count_lower,
                M::rinv_operate(&sum_upper, &sum_lower),
            )
        }
    }
}
use std::cmp::Reverse as BenchReverse;
use std::collections::BinaryHeap as BenchBinaryHeap;
use std::hint::black_box as bench_black_box;
use std::io::{self as bench_io, Read as BenchRead};
use std::time::Instant as BenchInstant;

const BENCH_SEED: u64 = 0x243f_6a88_85a3_08d3;
const BENCH_ROUNDS: usize = 5;

#[derive(Clone)]
struct BenchRng(u64);

impl BenchRng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    #[inline]
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 7;
        x ^= x >> 9;
        x ^= x << 8;
        self.0 = x;
        x
    }

    #[inline]
    fn index(&mut self, end: usize) -> usize {
        (self.next() as usize) % end
    }
}

#[inline]
fn bench_mix(state: u64, value: u64) -> u64 {
    state.rotate_left(17).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ value.wrapping_add(0x517c_c1b7_2722_0a95)
}

#[inline]
fn bench_integer_hash(value: u128) -> u64 {
    value as u64 ^ ((value >> 64) as u64).rotate_left(29)
}

fn bench_timed(mut operation: impl FnMut() -> u64) -> (u128, u64) {
    let start = BenchInstant::now();
    let checksum = bench_black_box(operation());
    (start.elapsed().as_nanos(), checksum)
}

type BenchCase<'a> = (&'a str, Box<dyn FnMut() -> (u128, u64) + 'a>);

fn bench_group(label: &str, units: usize, compare_checksums: bool, mut cases: Vec<BenchCase<'_>>) {
    assert!(!cases.is_empty());
    for (_, case) in &mut cases {
        let _ = case();
    }
    let mut samples = vec![[0_u128; BENCH_ROUNDS]; cases.len()];
    let mut checksums = vec![[0_u64; BENCH_ROUNDS]; cases.len()];
    for round in 0..BENCH_ROUNDS {
        if round % 2 == 0 {
            for index in 0..cases.len() {
                let (elapsed, checksum) = cases[index].1.as_mut()();
                samples[index][round] = elapsed;
                checksums[index][round] = checksum;
            }
        } else {
            for index in (0..cases.len()).rev() {
                let (elapsed, checksum) = cases[index].1.as_mut()();
                samples[index][round] = elapsed;
                checksums[index][round] = checksum;
            }
        }
    }
    let expected = checksums[0][0];
    for index in 0..cases.len() {
        assert!(
            checksums[index]
                .iter()
                .all(|&checksum| checksum == checksums[index][0])
        );
        if compare_checksums {
            assert_eq!(
                checksums[index][0], expected,
                "checksum mismatch in {label}"
            );
        }
        let mut sorted = samples[index];
        sorted.sort_unstable();
        let raw = samples[index]
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(",");
        println!(
            "case={label} impl={} units={units} raw_ns={raw} median_ns={} min_ns={} max_ns={} ns/unit={:.3} checksum={}",
            cases[index].0,
            sorted[BENCH_ROUNDS / 2],
            sorted[0],
            sorted[BENCH_ROUNDS - 1],
            sorted[BENCH_ROUNDS / 2] as f64 / units.max(1) as f64,
            checksums[index][0],
        );
    }
}

fn bench_backends() -> Vec<SimdBackend> {
    #[allow(unused_mut)]
    let mut result = vec![SimdBackend::Scalar];
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            result.push(SimdBackend::Avx2);
        }
        if avx512_supported() {
            result.push(SimdBackend::Avx512);
        }
    }
    result
}

fn bench_backend_name(backend: SimdBackend) -> &'static str {
    match backend {
        SimdBackend::Scalar => "scalar",
        SimdBackend::Avx2 => "avx2",
        SimdBackend::Avx512 => "avx512",
    }
}

fn bench_header(suite: usize) {
    #[cfg(target_arch = "x86_64")]
    let (bmi2, avx2, avx512f, avx512vl) = (
        is_x86_feature_detected!("bmi2"),
        is_x86_feature_detected!("avx2"),
        is_x86_feature_detected!("avx512f"),
        is_x86_feature_detected!("avx512vl"),
    );
    #[cfg(not(target_arch = "x86_64"))]
    let (bmi2, avx2, avx512f, avx512vl) = (false, false, false, false);
    println!(
        "benchmark=competitive_simd_methods_v2 suite={suite} seed={BENCH_SEED} rounds={BENCH_ROUNDS} warmups=1 order=alternating arch={} os={} pointer_bits={} bmi2={bmi2} avx2={avx2} avx512f={avx512f} avx512vl={avx512vl}",
        std::env::consts::ARCH,
        std::env::consts::OS,
        usize::BITS,
    );
    println!("build_flags=rustc_1.89.0 edition_2024 opt_level_3 lto_off target_cpu_default");
    #[cfg(target_arch = "x86_64")]
    #[allow(unused_unsafe)]
    unsafe {
        use std::arch::x86_64::{__cpuid, __cpuid_count};

        let leaf0 = __cpuid(0);
        let mut vendor = Vec::with_capacity(12);
        vendor.extend_from_slice(&leaf0.ebx.to_le_bytes());
        vendor.extend_from_slice(&leaf0.edx.to_le_bytes());
        vendor.extend_from_slice(&leaf0.ecx.to_le_bytes());
        let leaf1 = __cpuid(1);
        let base_family = (leaf1.eax >> 8) & 0x0f;
        let family = if base_family == 0x0f {
            base_family + ((leaf1.eax >> 20) & 0xff)
        } else {
            base_family
        };
        let base_model = (leaf1.eax >> 4) & 0x0f;
        let model = if base_family == 0x06 || base_family == 0x0f {
            base_model | (((leaf1.eax >> 16) & 0x0f) << 4)
        } else {
            base_model
        };
        let stepping = leaf1.eax & 0x0f;
        let mut brand = Vec::with_capacity(48);
        if __cpuid(0x8000_0000).eax >= 0x8000_0004 {
            for leaf in 0x8000_0002..=0x8000_0004 {
                let value = __cpuid_count(leaf, 0);
                brand.extend_from_slice(&value.eax.to_le_bytes());
                brand.extend_from_slice(&value.ebx.to_le_bytes());
                brand.extend_from_slice(&value.ecx.to_le_bytes());
                brand.extend_from_slice(&value.edx.to_le_bytes());
            }
        }
        println!(
            "cpu vendor={} family={} model={} stepping={} brand={}",
            String::from_utf8_lossy(&vendor),
            family,
            model,
            stepping,
            String::from_utf8_lossy(&brand)
                .trim_matches(char::from(0))
                .trim(),
        );
    }
}

fn bench_configure_runner() {
    #[cfg(target_os = "macos")]
    unsafe {
        unsafe extern "C" {
            fn pthread_set_qos_class_self_np(qos_class: u32, relative_priority: i32) -> i32;
        }

        const QOS_CLASS_USER_INTERACTIVE: u32 = 0x21;
        let result = pthread_set_qos_class_self_np(QOS_CLASS_USER_INTERACTIVE, 0);
        println!("runner_qos=user_interactive result={result}");
    }
}

fn bench_diagnostics() {
    let n = 1 << 20;
    let rounds = 32;
    let mut rng = BenchRng::new(BENCH_SEED);
    let data: Vec<u64> = (0..n).map(|_| rng.next()).collect();
    bench_group(
        "diagnostic_u64_memory_scan",
        n * rounds,
        true,
        vec![(
            "scalar",
            Box::new(|| {
                bench_timed(|| {
                    let mut checksum = 0;
                    for round in 0..rounds {
                        for &value in &data {
                            checksum = bench_mix(checksum, value ^ round as u64);
                        }
                    }
                    checksum
                })
            }),
        )],
    );

    for kibibytes in [4, 32, 256, 2_048, 8_192] {
        let entries = kibibytes * 1_024 / std::mem::size_of::<usize>();
        let mut order: Vec<usize> = (0..entries).collect();
        for index in (1..entries).rev() {
            let other = rng.index(index + 1);
            order.swap(index, other);
        }
        let mut links = vec![0; entries];
        for pair in order.windows(2) {
            links[pair[0]] = pair[1];
        }
        links[*order.last().unwrap()] = order[0];
        let pointer_steps = 4 * n;
        let label = format!("diagnostic_pointer_chase_{kibibytes}kib");
        bench_group(
            &label,
            pointer_steps,
            false,
            vec![(
                "scalar",
                Box::new(|| {
                    bench_timed(|| {
                        let mut index = order[0];
                        for _ in 0..pointer_steps {
                            index = unsafe { *links.get_unchecked(index) };
                        }
                        index as u64
                    })
                }),
            )],
        );
    }

    bench_simd_kernel_diagnostics();
}

fn bench_simd_kernel_diagnostics() {
    let q = 1_000_000;
    let mut rng = BenchRng::new(BENCH_SEED ^ 0xa000);
    let mut values32: [u32; 16] = std::array::from_fn(|_| rng.next() as u32);
    let mut values64: [u64; 8] = std::array::from_fn(|_| rng.next());
    values32.sort_unstable();
    values64.sort_unstable();
    let signed32 = values32.map(|value| value as i32);
    let signed64 = values64.map(|value| value as i64);

    macro_rules! kernel_cases {
        ($scalar:expr, $avx2:expr, $avx512:expr) => {{
            #[allow(unused_mut)]
            let mut cases: Vec<BenchCase<'_>> = vec![("scalar", Box::new($scalar))];
            #[cfg(target_arch = "x86_64")]
            {
                if is_x86_feature_detected!("avx2") {
                    cases.push(("avx2", Box::new($avx2)));
                }
                if avx512_supported() {
                    cases.push(("avx512", Box::new($avx512)));
                }
            }
            cases
        }};
    }

    bench_group(
        "diagnostic_first_gt_u32x16",
        q,
        true,
        kernel_cases!(
            || bench_timed(|| (0..q).fold(0, |sum, index| {
                let key = (index as u32).wrapping_mul(0x9e37_79b9);
                bench_mix(sum, values32.partition_point(|&value| value <= key) as u64)
            })),
            || bench_timed(|| (0..q).fold(0, |sum, index| {
                let key = (index as u32).wrapping_mul(0x9e37_79b9);
                let lane = unsafe { simd::first_gt_u32x16_avx2(&values32, key) };
                bench_mix(sum, lane as u64)
            })),
            || bench_timed(|| (0..q).fold(0, |sum, index| {
                let key = (index as u32).wrapping_mul(0x9e37_79b9);
                let lane = unsafe { simd::first_gt_u32x16_avx512(&values32, key) };
                bench_mix(sum, lane as u64)
            }))
        ),
    );
    bench_group(
        "diagnostic_first_gt_u64x8",
        q,
        true,
        kernel_cases!(
            || bench_timed(|| (0..q).fold(0, |sum, index| {
                let key = (index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
                bench_mix(sum, values64.partition_point(|&value| value <= key) as u64)
            })),
            || bench_timed(|| (0..q).fold(0, |sum, index| {
                let key = (index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
                let lane = unsafe { simd::first_gt_u64x8_avx2(&values64, key) };
                bench_mix(sum, lane as u64)
            })),
            || bench_timed(|| (0..q).fold(0, |sum, index| {
                let key = (index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
                let lane = unsafe { simd::first_gt_u64x8_avx512(&values64, key) };
                bench_mix(sum, lane as u64)
            }))
        ),
    );
    bench_group(
        "diagnostic_minimum_i32x16",
        q,
        true,
        kernel_cases!(
            || bench_timed(|| (0..q).fold(0, |sum, _| {
                bench_mix(
                    sum,
                    *bench_black_box(&signed32).iter().min().unwrap() as u64,
                )
            })),
            || bench_timed(|| (0..q).fold(0, |sum, _| {
                bench_mix(
                    sum,
                    unsafe { simd::minimum_i32x16_avx2(bench_black_box(&signed32)) } as u64,
                )
            })),
            || bench_timed(|| (0..q).fold(0, |sum, _| {
                bench_mix(
                    sum,
                    unsafe { simd::minimum_i32x16_avx512(bench_black_box(&signed32)) } as u64,
                )
            }))
        ),
    );
    bench_group(
        "diagnostic_minimum_i64x8",
        q,
        true,
        kernel_cases!(
            || bench_timed(|| (0..q).fold(0, |sum, _| {
                bench_mix(
                    sum,
                    *bench_black_box(&signed64).iter().min().unwrap() as u64,
                )
            })),
            || bench_timed(|| (0..q).fold(0, |sum, _| {
                bench_mix(
                    sum,
                    unsafe { simd::minimum_i64x8_avx2(bench_black_box(&signed64)) } as u64,
                )
            })),
            || bench_timed(|| (0..q).fold(0, |sum, _| {
                bench_mix(
                    sum,
                    unsafe { simd::minimum_i64x8_avx512(bench_black_box(&signed64)) } as u64,
                )
            }))
        ),
    );

    let q = q / 2;
    bench_group(
        "diagnostic_add_suffix_u32x16",
        q,
        true,
        kernel_cases!(
            || bench_timed(|| {
                let mut values = [0u32; 16];
                let mut checksum = 0;
                for index in 0..q {
                    let lane = index & 15;
                    let delta = (index as u32).wrapping_mul(0x9e37_79b9);
                    for value in &mut values[lane..] {
                        *value = value.wrapping_add(delta);
                    }
                    checksum = bench_mix(checksum, values[lane] as u64);
                }
                checksum
            }),
            || bench_timed(|| {
                let mut values = [0u32; 16];
                let mut checksum = 0;
                for index in 0..q {
                    let lane = index & 15;
                    let delta = (index as u32).wrapping_mul(0x9e37_79b9);
                    unsafe { simd::add_suffix_u32x16_avx2(&mut values, lane, delta) };
                    checksum = bench_mix(checksum, values[lane] as u64);
                }
                checksum
            }),
            || bench_timed(|| {
                let mut values = [0u32; 16];
                let mut checksum = 0;
                for index in 0..q {
                    let lane = index & 15;
                    let delta = (index as u32).wrapping_mul(0x9e37_79b9);
                    unsafe { simd::add_suffix_u32x16_avx512(&mut values, lane, delta) };
                    checksum = bench_mix(checksum, values[lane] as u64);
                }
                checksum
            })
        ),
    );
    bench_group(
        "diagnostic_add_suffix_u64x8",
        q,
        true,
        kernel_cases!(
            || bench_timed(|| {
                let mut values = [0u64; 8];
                let mut checksum = 0;
                for index in 0..q {
                    let lane = index & 7;
                    let delta = (index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
                    for value in &mut values[lane..] {
                        *value = value.wrapping_add(delta);
                    }
                    checksum = bench_mix(checksum, values[lane]);
                }
                checksum
            }),
            || bench_timed(|| {
                let mut values = [0u64; 8];
                let mut checksum = 0;
                for index in 0..q {
                    let lane = index & 7;
                    let delta = (index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
                    unsafe { simd::add_suffix_u64x8_avx2(&mut values, lane, delta) };
                    checksum = bench_mix(checksum, values[lane]);
                }
                checksum
            }),
            || bench_timed(|| {
                let mut values = [0u64; 8];
                let mut checksum = 0;
                for index in 0..q {
                    let lane = index & 7;
                    let delta = (index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
                    unsafe { simd::add_suffix_u64x8_avx512(&mut values, lane, delta) };
                    checksum = bench_mix(checksum, values[lane]);
                }
                checksum
            })
        ),
    );
    bench_group(
        "diagnostic_minimum_range_i32x16",
        q,
        true,
        kernel_cases!(
            || bench_timed(|| {
                (0..q).fold(0, |sum, index| {
                    let start = index & 15;
                    let end = start + 1 + (index / 16) % (16 - start);
                    bench_mix(
                        sum,
                        *bench_black_box(&signed32)[start..end].iter().min().unwrap() as u64,
                    )
                })
            }),
            || bench_timed(|| {
                (0..q).fold(0, |sum, index| {
                    let start = index & 15;
                    let end = start + 1 + (index / 16) % (16 - start);
                    bench_mix(
                        sum,
                        unsafe {
                            simd::minimum_range_i32x16_avx2(
                                bench_black_box(&signed32),
                                start,
                                end,
                            )
                        } as u64,
                    )
                })
            }),
            || bench_timed(|| {
                (0..q).fold(0, |sum, index| {
                    let start = index & 15;
                    let end = start + 1 + (index / 16) % (16 - start);
                    bench_mix(
                        sum,
                        unsafe {
                            simd::minimum_range_i32x16_avx512(
                                bench_black_box(&signed32),
                                start,
                                end,
                            )
                        } as u64,
                    )
                })
            })
        ),
    );
    bench_group(
        "diagnostic_minimum_range_i64x8",
        q,
        true,
        kernel_cases!(
            || bench_timed(|| {
                (0..q).fold(0, |sum, index| {
                    let start = index & 7;
                    let end = start + 1 + (index / 8) % (8 - start);
                    bench_mix(
                        sum,
                        *bench_black_box(&signed64)[start..end].iter().min().unwrap() as u64,
                    )
                })
            }),
            || bench_timed(|| {
                (0..q).fold(0, |sum, index| {
                    let start = index & 7;
                    let end = start + 1 + (index / 8) % (8 - start);
                    bench_mix(
                        sum,
                        unsafe {
                            simd::minimum_range_i64x8_avx2(
                                bench_black_box(&signed64),
                                start,
                                end,
                            )
                        } as u64,
                    )
                })
            }),
            || bench_timed(|| {
                (0..q).fold(0, |sum, index| {
                    let start = index & 7;
                    let end = start + 1 + (index / 8) % (8 - start);
                    bench_mix(
                        sum,
                        unsafe {
                            simd::minimum_range_i64x8_avx512(
                                bench_black_box(&signed64),
                                start,
                                end,
                            )
                        } as u64,
                    )
                })
            })
        ),
    );
    bench_group(
        "diagnostic_max_index_u32x16",
        q,
        true,
        kernel_cases!(
            || bench_timed(|| {
                (0..q).fold(0, |sum, _| {
                    let values = bench_black_box(&values32);
                    let mut lane = 0;
                    for index in 1..16 {
                        if values[index] > values[lane] {
                            lane = index;
                        }
                    }
                    bench_mix(sum, lane as u64)
                })
            }),
            || bench_timed(|| {
                (0..q).fold(0, |sum, _| {
                    bench_mix(
                        sum,
                        unsafe { simd::max_index_u32x16_avx2(bench_black_box(&values32)) } as u64,
                    )
                })
            }),
            || bench_timed(|| {
                (0..q).fold(0, |sum, _| {
                    bench_mix(
                        sum,
                        unsafe { simd::max_index_u32x16_avx512(bench_black_box(&values32)) } as u64,
                    )
                })
            })
        ),
    );
    bench_group(
        "diagnostic_max_index_u64x8",
        q,
        true,
        kernel_cases!(
            || bench_timed(|| {
                (0..q).fold(0, |sum, _| {
                    let values = bench_black_box(&values64);
                    let mut lane = 0;
                    for index in 1..8 {
                        if values[index] > values[lane] {
                            lane = index;
                        }
                    }
                    bench_mix(sum, lane as u64)
                })
            }),
            || bench_timed(|| {
                (0..q).fold(0, |sum, _| {
                    bench_mix(
                        sum,
                        unsafe { simd::max_index_u64x8_avx2(bench_black_box(&values64)) } as u64,
                    )
                })
            }),
            || bench_timed(|| {
                (0..q).fold(0, |sum, _| {
                    bench_mix(
                        sum,
                        unsafe { simd::max_index_u64x8_avx512(bench_black_box(&values64)) } as u64,
                    )
                })
            })
        ),
    );
}

fn bench_self_test() {
    let bits: BitVector = (0..10_003).map(|index| index % 7 == 0).collect();
    assert_eq!(bits.rank1(10_003), 1_429);
    assert_eq!(bits.select1(1_428), Some(9_996));
    let values: Vec<u32> = (0..10_003).map(|index| (index * 37 % 997) as u32).collect();
    let wavelet = WaveletMatrix::new(values.clone());
    assert_eq!(wavelet.access(123), values[123]);
    let mut sorted = values.clone();
    sorted.sort_unstable();
    let search = StaticSearch::from_sorted(&sorted);
    assert_eq!(
        search.position_bisect(500),
        sorted.partition_point(|&value| value < 500)
    );
    let mut heap = DaryHeapU32::from(values.clone());
    let mut binary = BenchBinaryHeap::from(values.clone());
    for _ in 0..1_000 {
        assert_eq!(heap.pop(), binary.pop());
    }
    let prefix_values: Vec<u32> = values.iter().map(|value| value % 8).collect();
    let prefix = WidePrefixU32::from_slice(&prefix_values);
    assert_eq!(prefix.fold(100, 900), prefix_values[100..900].iter().sum());
    let tree = WideSegmentTreeMinI32::from_vec(values.iter().map(|&value| value as i32).collect());
    assert_eq!(
        tree.fold(100..900),
        values[100..900].iter().copied().min().unwrap() as i32
    );
    let mut radix = RadixHeapU64::new();
    for index in 0..1_000 {
        radix.push(index / 3, index);
    }
    let mut last = 0;
    while let Some((key, _)) = radix.pop() {
        assert!(last <= key);
        last = key;
    }
    bench_radix_signed_self_test();
    println!("self_test=ok");
}

#[derive(Clone, Debug)]
struct BenchLegacyBitVector {
    data: Vec<(u64, usize)>,
    sum: usize,
    len: usize,
}

impl BenchLegacyBitVector {
    fn with_capacity(bits: usize) -> Self {
        let mut data = Vec::with_capacity(bits.div_ceil(64) + 1);
        data.push((0, 0));
        Self {
            data,
            sum: 0,
            len: 0,
        }
    }

    fn push(&mut self, bit: bool) {
        let word = self.len / 64;
        let offset = self.len % 64;
        if word == self.data.len() - 1 {
            self.data.push((0, self.sum));
        }
        if bit {
            self.data[word].0 |= 1 << offset;
            self.sum += 1;
        }
        self.len += 1;
        self.data.last_mut().unwrap().1 = self.sum;
    }

    #[inline]
    fn access(&self, index: usize) -> bool {
        self.data[index / 64].0 >> (index % 64) & 1 != 0
    }

    #[inline]
    fn rank1(&self, end: usize) -> usize {
        let word = end / 64;
        let offset = end % 64;
        self.data[word].1 + (self.data[word].0 & !(u64::MAX << offset)).count_ones() as usize
    }

    #[inline]
    fn access_rank1(&self, index: usize) -> (bool, usize) {
        (self.access(index), self.rank1(index))
    }

    #[inline]
    fn rank0(&self, end: usize) -> usize {
        end - self.rank1(end)
    }

    fn select1(&self, mut rank: usize) -> Option<usize> {
        if self.sum <= rank {
            return None;
        }
        let mut left = 0;
        let mut right = self.data.len();
        while right - left > 1 {
            let middle = left.midpoint(right);
            if self.data[middle].1 <= rank {
                left = middle;
            } else {
                right = middle;
            }
        }
        let (word, prefix) = self.data[left];
        rank -= prefix;
        Some(left * 64 + word.select1(rank).unwrap())
    }

    fn select0(&self, mut rank: usize) -> Option<usize> {
        if self.len - self.sum <= rank {
            return None;
        }
        let mut left = 0;
        let mut right = self.data.len();
        while right - left > 1 {
            let middle = left.midpoint(right);
            if middle * 64 - self.data[middle].1 <= rank {
                left = middle;
            } else {
                right = middle;
            }
        }
        let (word, prefix) = self.data[left];
        rank -= left * 64 - prefix;
        Some(left * 64 + (!word).select1(rank).unwrap())
    }
}

impl FromIterator<bool> for BenchLegacyBitVector {
    fn from_iter<I: IntoIterator<Item = bool>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (lower, upper) = iter.size_hint();
        let mut data = Vec::with_capacity(upper.unwrap_or(lower).div_ceil(64) + 1);
        let mut word = 0u64;
        let mut word_len = 0;
        let mut sum = 0;
        let mut len = 0;
        for bit in iter {
            word |= (bit as u64) << word_len;
            word_len += 1;
            len += 1;
            if word_len == 64 {
                data.push((word, sum));
                sum += word.count_ones() as usize;
                word = 0;
                word_len = 0;
            }
        }
        if word_len != 0 {
            data.push((word, sum));
            sum += word.count_ones() as usize;
        }
        data.push((0, sum));
        Self { data, sum, len }
    }
}

fn bench_make_bits(n: usize, one_in: u64) -> Vec<bool> {
    let mut rng = BenchRng::new(BENCH_SEED ^ one_in);
    (0..n).map(|_| rng.next() % one_in == 0).collect()
}

fn bench_bit_vector_queries() {
    let n = (1 << 20) + 123;
    let q = 300_000;
    for one_in in [2, 64] {
        let bits = bench_make_bits(n, one_in);
        let mut current = BitVector::with_capacity(n);
        let mut legacy = BenchLegacyBitVector::with_capacity(n);
        for &bit in &bits {
            current.push(bit);
            legacy.push(bit);
        }
        let mut rng = BenchRng::new(BENCH_SEED ^ n as u64 ^ one_in);
        let indices: Vec<_> = (0..q).map(|_| rng.index(n)).collect();
        let ends: Vec<_> = (0..q).map(|_| rng.index(n + 1)).collect();
        let label = format!("bit_vector_density_1_in_{one_in}_access");
        bench_group(
            &label,
            q,
            true,
            vec![
                (
                    "legacy",
                    Box::new(|| {
                        bench_timed(|| {
                            indices
                                .iter()
                                .fold(0, |sum, &index| bench_mix(sum, legacy.access(index) as u64))
                        })
                    }),
                ),
                (
                    "current",
                    Box::new(|| {
                        bench_timed(|| {
                            indices.iter().fold(0, |sum, &index| {
                                bench_mix(sum, current.access(index) as u64)
                            })
                        })
                    }),
                ),
            ],
        );
        let label = format!("bit_vector_density_1_in_{one_in}_access_rank1");
        bench_group(
            &label,
            q,
            true,
            vec![
                (
                    "legacy",
                    Box::new(|| {
                        bench_timed(|| {
                            indices.iter().fold(0, |sum, &index| {
                                let (bit, rank) = legacy.access_rank1(index);
                                bench_mix(sum, rank as u64 ^ ((bit as u64) << 63))
                            })
                        })
                    }),
                ),
                (
                    "current",
                    Box::new(|| {
                        bench_timed(|| {
                            indices.iter().fold(0, |sum, &index| {
                                let (bit, rank) = current.access_rank1(index);
                                bench_mix(sum, rank as u64 ^ ((bit as u64) << 63))
                            })
                        })
                    }),
                ),
            ],
        );
        for rank0 in [false, true] {
            let label = format!(
                "bit_vector_density_1_in_{one_in}_{}",
                if rank0 { "rank0" } else { "rank1" }
            );
            bench_group(
                &label,
                q,
                true,
                vec![
                    (
                        "legacy",
                        Box::new(|| {
                            bench_timed(|| {
                                ends.iter().fold(0, |sum, &end| {
                                    bench_mix(
                                        sum,
                                        if rank0 {
                                            legacy.rank0(end)
                                        } else {
                                            legacy.rank1(end)
                                        } as u64,
                                    )
                                })
                            })
                        }),
                    ),
                    (
                        "current",
                        Box::new(|| {
                            bench_timed(|| {
                                ends.iter().fold(0, |sum, &end| {
                                    bench_mix(
                                        sum,
                                        if rank0 {
                                            current.rank0(end)
                                        } else {
                                            current.rank1(end)
                                        } as u64,
                                    )
                                })
                            })
                        }),
                    ),
                ],
            );
        }
    }
}

fn bench_bit_vector_build_select() {
    let n = (1 << 20) + 123;
    let q = 200_000;
    for one_in in [2, 64] {
        let bits = bench_make_bits(n, one_in);
        let label = format!("bit_vector_density_1_in_{one_in}_build_collect");
        bench_group(
            &label,
            n,
            false,
            vec![
                (
                    "legacy_push",
                    Box::new(|| {
                        let values = bits.clone();
                        let mut vector = BenchLegacyBitVector::with_capacity(values.len());
                        let start = BenchInstant::now();
                        for bit in values {
                            vector.push(bit);
                        }
                        bench_black_box(&vector);
                        let elapsed = start.elapsed().as_nanos();
                        (elapsed, bench_mix(vector.sum as u64, vector.len as u64))
                    }),
                ),
                (
                    "current_collect",
                    Box::new(|| {
                        let values = bits.clone();
                        let start = BenchInstant::now();
                        let vector: BitVector = values.into_iter().collect();
                        bench_black_box(&vector);
                        let elapsed = start.elapsed().as_nanos();
                        (
                            elapsed,
                            bench_mix(
                                vector.rank1(vector.bit_length()) as u64,
                                vector.bit_length() as u64,
                            ),
                        )
                    }),
                ),
                (
                    "current_push",
                    Box::new(|| {
                        let values = bits.clone();
                        let mut vector = BitVector::with_capacity(values.len());
                        let start = BenchInstant::now();
                        for bit in values {
                            vector.push(bit);
                        }
                        bench_black_box(&vector);
                        let elapsed = start.elapsed().as_nanos();
                        (
                            elapsed,
                            bench_mix(
                                vector.rank1(vector.bit_length()) as u64,
                                vector.bit_length() as u64,
                            ),
                        )
                    }),
                ),
            ],
        );

        let legacy: BenchLegacyBitVector = bits.iter().copied().collect();
        let current: BitVector = bits.iter().copied().collect();
        let ones = current.rank1(n);
        let zeros = n - ones;
        let mut rng = BenchRng::new(BENCH_SEED ^ one_in ^ 0x55aa);
        let ranks1: Vec<_> = (0..q).map(|_| rng.index(ones)).collect();
        let ranks0: Vec<_> = (0..q).map(|_| rng.index(zeros)).collect();
        for select_zero in [false, true] {
            let ranks = if select_zero { &ranks0 } else { &ranks1 };
            let label = format!(
                "bit_vector_density_1_in_{one_in}_{}",
                if select_zero { "select0" } else { "select1" }
            );
            let scalar = current.clone().benchmark_scalar();
            #[allow(unused_mut)]
            let mut cases: Vec<BenchCase<'_>> = vec![
                (
                    "legacy",
                    Box::new(|| {
                        bench_timed(|| {
                            ranks.iter().fold(0, |sum, &rank| {
                                let index = if select_zero {
                                    legacy.select0(rank)
                                } else {
                                    legacy.select1(rank)
                                }
                                .unwrap();
                                bench_mix(sum, index as u64)
                            })
                        })
                    }),
                ),
                (
                    "scalar",
                    Box::new(|| {
                        bench_timed(|| {
                            ranks.iter().fold(0, |sum, &rank| {
                                let index = if select_zero {
                                    scalar.select0(rank)
                                } else {
                                    scalar.select1(rank)
                                }
                                .unwrap();
                                bench_mix(sum, index as u64)
                            })
                        })
                    }),
                ),
            ];
            #[cfg(target_arch = "x86_64")]
            if is_x86_feature_detected!("bmi2") {
                let bmi2 = current.clone().benchmark_bmi2();
                cases.push((
                    "bmi2",
                    Box::new(move || {
                        bench_timed(|| {
                            ranks.iter().fold(0, |sum, &rank| {
                                let index = if select_zero {
                                    bmi2.select0(rank)
                                } else {
                                    bmi2.select1(rank)
                                }
                                .unwrap();
                                bench_mix(sum, index as u64)
                            })
                        })
                    }),
                ));
            }
            bench_group(&label, q, true, cases);
        }
    }
}

fn bench_wavelet_data(n: usize, alphabet: u32) -> Vec<u32> {
    let mut rng = BenchRng::new(BENCH_SEED ^ n as u64 ^ alphabet as u64);
    (0..n).map(|_| (rng.next() as u32) % alphabet).collect()
}

fn bench_wavelet_ranges(n: usize, q: usize, seed: u64) -> Vec<std::ops::Range<usize>> {
    let mut rng = BenchRng::new(seed);
    (0..q)
        .map(|_| {
            let left = rng.index(n);
            let length = 1 + rng.index((n - left).min(4096));
            left..left + length
        })
        .collect()
}

fn bench_wavelet_queries() {
    let n = (1 << 18) + 123;
    let q = 120_000;
    let values = bench_wavelet_data(n, 1 << 16);
    bench_group(
        "wavelet_build_new",
        n,
        false,
        vec![
            (
                "legacy",
                Box::new(|| {
                    let input = values.clone();
                    let start = BenchInstant::now();
                    let matrix = bench_legacy_wavelet::WaveletMatrix::new(input);
                    bench_black_box(&matrix);
                    let elapsed = start.elapsed().as_nanos();
                    (
                        elapsed,
                        bench_mix(matrix.access(n / 3) as u64, matrix.access(2 * n / 3) as u64),
                    )
                }),
            ),
            (
                "current",
                Box::new(|| {
                    let input = values.clone();
                    let start = BenchInstant::now();
                    let matrix = WaveletMatrix::new(input);
                    bench_black_box(&matrix);
                    let elapsed = start.elapsed().as_nanos();
                    (
                        elapsed,
                        bench_mix(matrix.access(n / 3) as u64, matrix.access(2 * n / 3) as u64),
                    )
                }),
            ),
        ],
    );
    bench_group(
        "wavelet_build_new_with_init",
        n,
        false,
        vec![
            (
                "legacy",
                Box::new(|| {
                    let input = values.clone();
                    let mut checksum = 0;
                    let start = BenchInstant::now();
                    let matrix = bench_legacy_wavelet::WaveletMatrix::new_with_init(
                        input,
                        |depth, index, value| {
                            checksum = bench_mix(
                                checksum,
                                depth as u64 ^ (index as u64).rotate_left(11) ^ value as u64,
                            );
                        },
                    );
                    bench_black_box(&matrix);
                    let elapsed = start.elapsed().as_nanos();
                    (elapsed, bench_mix(checksum, matrix.access(n / 2) as u64))
                }),
            ),
            (
                "current",
                Box::new(|| {
                    let input = values.clone();
                    let mut checksum = 0;
                    let start = BenchInstant::now();
                    let matrix = WaveletMatrix::new_with_init(input, |depth, index, value| {
                        checksum = bench_mix(
                            checksum,
                            depth as u64 ^ (index as u64).rotate_left(11) ^ value as u64,
                        );
                    });
                    bench_black_box(&matrix);
                    let elapsed = start.elapsed().as_nanos();
                    (elapsed, bench_mix(checksum, matrix.access(n / 2) as u64))
                }),
            ),
        ],
    );

    let legacy = bench_legacy_wavelet::WaveletMatrix::new(values.clone());
    let matrix = WaveletMatrix::new(values.clone());
    let mut rng = BenchRng::new(BENCH_SEED ^ 0x1000);
    let indices: Vec<_> = (0..q).map(|_| rng.index(n)).collect();
    bench_group(
        "wavelet_access",
        q,
        false,
        vec![
            (
                "legacy",
                Box::new(|| {
                    bench_timed(|| {
                        indices
                            .iter()
                            .fold(0, |sum, &index| bench_mix(sum, legacy.access(index) as u64))
                    })
                }),
            ),
            (
                "current",
                Box::new(|| {
                    bench_timed(|| {
                        indices
                            .iter()
                            .fold(0, |sum, &index| bench_mix(sum, matrix.access(index) as u64))
                    })
                }),
            ),
        ],
    );

    let ranges = bench_wavelet_ranges(n, q, BENCH_SEED ^ 0x2000);
    let hit_values: Vec<_> = indices.iter().map(|&index| values[index]).collect();
    bench_group(
        "wavelet_rank_hit",
        q,
        false,
        vec![
            (
                "legacy",
                Box::new(|| {
                    bench_timed(|| {
                        hit_values
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                bench_mix(sum, legacy.rank(value, range.clone()) as u64)
                            })
                    })
                }),
            ),
            (
                "current",
                Box::new(|| {
                    bench_timed(|| {
                        hit_values
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                bench_mix(sum, matrix.rank(value, range.clone()) as u64)
                            })
                    })
                }),
            ),
        ],
    );
    bench_group(
        "wavelet_rank_miss",
        q,
        false,
        vec![
            (
                "legacy",
                Box::new(|| {
                    bench_timed(|| {
                        ranges.iter().fold(0, |sum, range| {
                            bench_mix(sum, legacy.rank(u32::MAX, range.clone()) as u64)
                        })
                    })
                }),
            ),
            (
                "current",
                Box::new(|| {
                    bench_timed(|| {
                        ranges.iter().fold(0, |sum, range| {
                            bench_mix(sum, matrix.rank(u32::MAX, range.clone()) as u64)
                        })
                    })
                }),
            ),
        ],
    );

    let quantile_k: Vec<_> = ranges.iter().map(|range| rng.index(range.len())).collect();
    bench_group(
        "wavelet_quantile",
        q,
        false,
        vec![
            (
                "legacy",
                Box::new(|| {
                    bench_timed(|| {
                        ranges.iter().zip(&quantile_k).fold(0, |sum, (range, &k)| {
                            bench_mix(sum, legacy.quantile(range.clone(), k) as u64)
                        })
                    })
                }),
            ),
            (
                "current",
                Box::new(|| {
                    bench_timed(|| {
                        ranges.iter().zip(&quantile_k).fold(0, |sum, (range, &k)| {
                            bench_mix(sum, matrix.quantile(range.clone(), k) as u64)
                        })
                    })
                }),
            ),
        ],
    );

    let outer_ranges: Vec<_> = (0..q)
        .map(|_| {
            let left = n / 4 + rng.index(n / 8);
            let length = 1 + rng.index(n / 8);
            left..left + length
        })
        .collect();
    let outer_k: Vec<_> = outer_ranges
        .iter()
        .map(|range| rng.index(n - range.len()))
        .collect();
    bench_group(
        "wavelet_quantile_outer",
        q,
        false,
        vec![
            (
                "legacy",
                Box::new(|| {
                    bench_timed(|| {
                        outer_ranges
                            .iter()
                            .zip(&outer_k)
                            .fold(0, |sum, (range, &k)| {
                                bench_mix(sum, legacy.quantile_outer(range.clone(), k) as u64)
                            })
                    })
                }),
            ),
            (
                "current",
                Box::new(|| {
                    bench_timed(|| {
                        outer_ranges
                            .iter()
                            .zip(&outer_k)
                            .fold(0, |sum, (range, &k)| {
                                bench_mix(sum, matrix.quantile_outer(range.clone(), k) as u64)
                            })
                    })
                }),
            ),
        ],
    );

    let thresholds: Vec<_> = (0..q).map(|_| rng.next() as u32 % (1 << 16)).collect();
    bench_group(
        "wavelet_rank_lessthan",
        q,
        false,
        vec![
            (
                "legacy",
                Box::new(|| {
                    bench_timed(|| {
                        thresholds
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                bench_mix(sum, legacy.rank_lessthan(value, range.clone()) as u64)
                            })
                    })
                }),
            ),
            (
                "current",
                Box::new(|| {
                    bench_timed(|| {
                        thresholds
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                bench_mix(sum, matrix.rank_lessthan(value, range.clone()) as u64)
                            })
                    })
                }),
            ),
        ],
    );
    bench_group(
        "wavelet_rank_range",
        q,
        false,
        vec![
            (
                "legacy",
                Box::new(|| {
                    bench_timed(|| {
                        thresholds
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                let end = value.saturating_add(1024);
                                bench_mix(sum, legacy.rank_range(value..end, range.clone()) as u64)
                            })
                    })
                }),
            ),
            (
                "current",
                Box::new(|| {
                    bench_timed(|| {
                        thresholds
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                let end = value.saturating_add(1024);
                                bench_mix(sum, matrix.rank_range(value..end, range.clone()) as u64)
                            })
                    })
                }),
            ),
        ],
    );
    bench_group(
        "wavelet_query_less_than",
        q,
        false,
        vec![
            (
                "legacy",
                Box::new(|| {
                    bench_timed(|| {
                        thresholds
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                let mut local = 0;
                                legacy.query_less_than(value, range.clone(), |depth, part| {
                                    local = bench_mix(
                                        local,
                                        depth as u64
                                            ^ ((part.start as u64) << 32)
                                            ^ part.end as u64,
                                    );
                                });
                                bench_mix(sum, local)
                            })
                    })
                }),
            ),
            (
                "current",
                Box::new(|| {
                    bench_timed(|| {
                        thresholds
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                let mut local = 0;
                                matrix.query_less_than(value, range.clone(), |depth, part| {
                                    local = bench_mix(
                                        local,
                                        depth as u64
                                            ^ ((part.start as u64) << 32)
                                            ^ part.end as u64,
                                    );
                                });
                                bench_mix(sum, local)
                            })
                    })
                }),
            ),
        ],
    );

    let mut occurrence = std::collections::HashMap::new();
    let mut rank_at = Vec::with_capacity(n);
    for &value in &values {
        let count = occurrence.entry(value).or_insert(0_usize);
        rank_at.push(*count);
        *count += 1;
    }
    let select_queries: Vec<_> = indices
        .iter()
        .map(|&index| (values[index], rank_at[index]))
        .collect();
    let scalar = matrix.clone().benchmark_scalar();
    #[allow(unused_mut)]
    let mut cases: Vec<BenchCase<'_>> = vec![
        (
            "legacy",
            Box::new(|| {
                bench_timed(|| {
                    select_queries.iter().fold(0, |sum, &(value, rank)| {
                        bench_mix(sum, legacy.select(value, rank).unwrap() as u64)
                    })
                })
            }),
        ),
        (
            "scalar",
            Box::new(|| {
                bench_timed(|| {
                    select_queries.iter().fold(0, |sum, &(value, rank)| {
                        bench_mix(sum, scalar.select(value, rank).unwrap() as u64)
                    })
                })
            }),
        ),
    ];
    #[cfg(target_arch = "x86_64")]
    if is_x86_feature_detected!("bmi2") {
        let bmi2 = matrix.clone().benchmark_bmi2();
        let select_queries = &select_queries;
        cases.push((
            "bmi2",
            Box::new(move || {
                bench_timed(|| {
                    select_queries.iter().fold(0, |sum, &(value, rank)| {
                        bench_mix(sum, bmi2.select(value, rank).unwrap() as u64)
                    })
                })
            }),
        ));
    }
    bench_group("wavelet_select_hit", q, true, cases);

    let select_miss_queries: Vec<_> = indices
        .iter()
        .map(|&index| {
            let value = values[index];
            (value, occurrence[&value])
        })
        .collect();
    let scalar = matrix.clone().benchmark_scalar();
    #[allow(unused_mut)]
    let mut cases: Vec<BenchCase<'_>> = vec![
        (
            "legacy",
            Box::new(|| {
                bench_timed(|| {
                    select_miss_queries.iter().fold(0, |sum, &(value, rank)| {
                        bench_mix(sum, legacy.select(value, rank).is_none() as u64)
                    })
                })
            }),
        ),
        (
            "scalar",
            Box::new(|| {
                bench_timed(|| {
                    select_miss_queries.iter().fold(0, |sum, &(value, rank)| {
                        bench_mix(sum, scalar.select(value, rank).is_none() as u64)
                    })
                })
            }),
        ),
    ];
    #[cfg(target_arch = "x86_64")]
    if is_x86_feature_detected!("bmi2") {
        let bmi2 = matrix.benchmark_bmi2();
        let select_miss_queries = &select_miss_queries;
        cases.push((
            "bmi2",
            Box::new(move || {
                bench_timed(|| {
                    select_miss_queries.iter().fold(0, |sum, &(value, rank)| {
                        bench_mix(sum, bmi2.select(value, rank).is_none() as u64)
                    })
                })
            }),
        ));
    }
    bench_group("wavelet_select_miss", q, true, cases);
}

fn bench_wavelet_batch() {
    let n = (1 << 18) + 123;
    let q = 131_072;
    let values = bench_wavelet_data(n, 1 << 16);
    let legacy = bench_legacy_wavelet::WaveletMatrix::new(values.clone());
    let matrix = WaveletMatrix::new(values.clone());
    let mut rng = BenchRng::new(BENCH_SEED ^ 0x3000);
    let indices: Vec<_> = (0..q).map(|_| rng.index(n)).collect();
    let ranges = bench_wavelet_ranges(n, q, BENCH_SEED ^ 0x4000);
    let rank_queries: Vec<_> = indices
        .iter()
        .zip(&ranges)
        .map(|(&index, range)| (values[index], range.clone()))
        .collect();
    let quantile_queries: Vec<_> = ranges
        .iter()
        .map(|range| (range.clone(), rng.index(range.len())))
        .collect();
    for batch in [1, 4, 8, 16, 64] {
        let label = format!("wavelet_access_batch_{batch}");
        bench_group(
            &label,
            q,
            false,
            vec![
                (
                    "current_loop",
                    Box::new(|| {
                        bench_timed(|| {
                            indices
                                .iter()
                                .fold(0, |sum, &index| bench_mix(sum, matrix.access(index) as u64))
                        })
                    }),
                ),
                (
                    "current_batch",
                    Box::new(|| {
                        bench_timed(|| {
                            indices.chunks(batch).fold(0, |mut sum, chunk| {
                                for value in matrix.access_batch(chunk.iter().copied()) {
                                    sum = bench_mix(sum, value as u64);
                                }
                                sum
                            })
                        })
                    }),
                ),
            ],
        );
        let label = format!("wavelet_rank_batch_{batch}");
        bench_group(
            &label,
            q,
            false,
            vec![
                (
                    "current_loop",
                    Box::new(|| {
                        bench_timed(|| {
                            rank_queries.iter().fold(0, |sum, (value, range)| {
                                bench_mix(sum, matrix.rank(*value, range.clone()) as u64)
                            })
                        })
                    }),
                ),
                (
                    "current_batch",
                    Box::new(|| {
                        bench_timed(|| {
                            rank_queries.chunks(batch).fold(0, |mut sum, chunk| {
                                for value in matrix.rank_batch(chunk.iter().cloned()) {
                                    sum = bench_mix(sum, value as u64);
                                }
                                sum
                            })
                        })
                    }),
                ),
            ],
        );
        let label = format!("wavelet_quantile_batch_{batch}");
        bench_group(
            &label,
            q,
            false,
            vec![
                (
                    "legacy_batch",
                    Box::new(|| {
                        bench_timed(|| {
                            quantile_queries.chunks(batch).fold(0, |mut sum, chunk| {
                                for value in legacy.quantile_batch(chunk.iter().cloned()) {
                                    sum = bench_mix(sum, value as u64);
                                }
                                sum
                            })
                        })
                    }),
                ),
                (
                    "current_loop",
                    Box::new(|| {
                        bench_timed(|| {
                            quantile_queries.iter().fold(0, |sum, (range, k)| {
                                bench_mix(sum, matrix.quantile(range.clone(), *k) as u64)
                            })
                        })
                    }),
                ),
                (
                    "current_batch",
                    Box::new(|| {
                        bench_timed(|| {
                            quantile_queries.chunks(batch).fold(0, |mut sum, chunk| {
                                for value in matrix.quantile_batch(chunk.iter().cloned()) {
                                    sum = bench_mix(sum, value as u64);
                                }
                                sum
                            })
                        })
                    }),
                ),
            ],
        );
    }
}

fn bench_wavelet_fold() {
    let n = (1 << 17) + 123;
    let q = 80_000;
    let values = bench_wavelet_data(n, 1 << 15);
    let weights: Vec<i64> = values.iter().map(|&value| value as i64 % 1000).collect();
    let legacy_matrix = bench_legacy_wavelet::WaveletMatrix::new(values.clone());
    let matrix = WaveletMatrix::new(values.clone());
    bench_group(
        "wavelet_fold_build",
        n,
        false,
        vec![
            (
                "legacy",
                Box::new(|| {
                    let input = weights.clone();
                    let start = BenchInstant::now();
                    let fold = legacy_matrix.build_fold::<AdditiveOperation<i64>>(&input);
                    bench_black_box(&fold);
                    let elapsed = start.elapsed().as_nanos();
                    (elapsed, fold.fold_lessthan(u32::MAX, 0..n) as u64)
                }),
            ),
            (
                "current",
                Box::new(|| {
                    let input = weights.clone();
                    let start = BenchInstant::now();
                    let fold = matrix.build_fold::<AdditiveOperation<i64>>(&input);
                    bench_black_box(&fold);
                    let elapsed = start.elapsed().as_nanos();
                    (elapsed, fold.fold_lessthan(u32::MAX, 0..n) as u64)
                }),
            ),
        ],
    );
    bench_group(
        "wavelet_point_add_build",
        n,
        false,
        vec![
            (
                "legacy",
                Box::new(|| {
                    let input = weights.clone();
                    let start = BenchInstant::now();
                    let point = legacy_matrix.build_point_add::<AdditiveOperation<i64>>(&input);
                    bench_black_box(&point);
                    let elapsed = start.elapsed().as_nanos();
                    (elapsed, point.fold_lessthan(u32::MAX, 0..n) as u64)
                }),
            ),
            (
                "current",
                Box::new(|| {
                    let input = weights.clone();
                    let start = BenchInstant::now();
                    let point = matrix.build_point_add::<AdditiveOperation<i64>>(&input);
                    bench_black_box(&point);
                    let elapsed = start.elapsed().as_nanos();
                    (elapsed, point.fold_lessthan(u32::MAX, 0..n) as u64)
                }),
            ),
        ],
    );
    let legacy_fold = legacy_matrix.build_fold::<AdditiveOperation<i64>>(&weights);
    let legacy_point = legacy_matrix.build_point_add::<AdditiveOperation<i64>>(&weights);
    let fold = matrix.build_fold::<AdditiveOperation<i64>>(&weights);
    let point = matrix.build_point_add::<AdditiveOperation<i64>>(&weights);
    let ranges = bench_wavelet_ranges(n, q, BENCH_SEED ^ 0x5000);
    let mut rng = BenchRng::new(BENCH_SEED ^ 0x6000);
    let thresholds: Vec<_> = (0..q).map(|_| rng.next() as u32 % (1 << 15)).collect();
    bench_group(
        "wavelet_fold_lessthan",
        q,
        false,
        vec![
            (
                "legacy_static",
                Box::new(|| {
                    bench_timed(|| {
                        thresholds
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                bench_mix(
                                    sum,
                                    legacy_fold.fold_lessthan(value, range.clone()) as u64,
                                )
                            })
                    })
                }),
            ),
            (
                "current_static",
                Box::new(|| {
                    bench_timed(|| {
                        thresholds
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                bench_mix(sum, fold.fold_lessthan(value, range.clone()) as u64)
                            })
                    })
                }),
            ),
        ],
    );
    bench_group(
        "wavelet_fold_lessthan_with_count",
        q,
        false,
        vec![
            (
                "legacy_static",
                Box::new(|| {
                    bench_timed(|| {
                        thresholds
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                let (count, value) =
                                    legacy_fold.fold_lessthan_with_count(value, range.clone());
                                bench_mix(sum, value as u64 ^ (count as u64).rotate_left(23))
                            })
                    })
                }),
            ),
            (
                "current_static",
                Box::new(|| {
                    bench_timed(|| {
                        thresholds
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                let (count, value) =
                                    fold.fold_lessthan_with_count(value, range.clone());
                                bench_mix(sum, value as u64 ^ (count as u64).rotate_left(23))
                            })
                    })
                }),
            ),
        ],
    );
    bench_group(
        "wavelet_fold_range",
        q,
        false,
        vec![
            (
                "legacy_static",
                Box::new(|| {
                    bench_timed(|| {
                        thresholds
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                bench_mix(
                                    sum,
                                    legacy_fold.fold_range(
                                        value..value.saturating_add(1024),
                                        range.clone(),
                                    ) as u64,
                                )
                            })
                    })
                }),
            ),
            (
                "current_static",
                Box::new(|| {
                    bench_timed(|| {
                        thresholds
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                bench_mix(
                                    sum,
                                    fold.fold_range(
                                        value..value.saturating_add(1024),
                                        range.clone(),
                                    ) as u64,
                                )
                            })
                    })
                }),
            ),
            (
                "legacy_point_add",
                Box::new(|| {
                    bench_timed(|| {
                        thresholds
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                bench_mix(
                                    sum,
                                    legacy_point.fold_range(
                                        value..value.saturating_add(1024),
                                        range.clone(),
                                    ) as u64,
                                )
                            })
                    })
                }),
            ),
            (
                "current_point_add",
                Box::new(|| {
                    bench_timed(|| {
                        thresholds
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                bench_mix(
                                    sum,
                                    point.fold_range(
                                        value..value.saturating_add(1024),
                                        range.clone(),
                                    ) as u64,
                                )
                            })
                    })
                }),
            ),
        ],
    );
    bench_group(
        "wavelet_fold_range_with_count",
        q,
        false,
        vec![
            (
                "legacy_static",
                Box::new(|| {
                    bench_timed(|| {
                        thresholds
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                let (count, value) = legacy_fold.fold_range_with_count(
                                    value..value.saturating_add(1024),
                                    range.clone(),
                                );
                                bench_mix(sum, value as u64 ^ (count as u64).rotate_left(23))
                            })
                    })
                }),
            ),
            (
                "current_static",
                Box::new(|| {
                    bench_timed(|| {
                        thresholds
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                let (count, value) = fold.fold_range_with_count(
                                    value..value.saturating_add(1024),
                                    range.clone(),
                                );
                                bench_mix(sum, value as u64 ^ (count as u64).rotate_left(23))
                            })
                    })
                }),
            ),
        ],
    );
    bench_group(
        "wavelet_point_add_fold_lessthan",
        q,
        false,
        vec![
            (
                "legacy_point_add",
                Box::new(|| {
                    bench_timed(|| {
                        thresholds
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                bench_mix(
                                    sum,
                                    legacy_point.fold_lessthan(value, range.clone()) as u64,
                                )
                            })
                    })
                }),
            ),
            (
                "current_point_add",
                Box::new(|| {
                    bench_timed(|| {
                        thresholds
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                bench_mix(sum, point.fold_lessthan(value, range.clone()) as u64)
                            })
                    })
                }),
            ),
        ],
    );
    let update_indices: Vec<_> = (0..q).map(|_| rng.index(n)).collect();
    let update_values: Vec<_> = (0..q).map(|_| (rng.next() % 100) as i64).collect();
    bench_group(
        "wavelet_point_add_update",
        q,
        false,
        vec![
            (
                "legacy_point_add",
                Box::new(|| {
                    let mut point =
                        legacy_matrix.build_point_add::<AdditiveOperation<i64>>(&weights);
                    let start = BenchInstant::now();
                    for (&index, &value) in update_indices.iter().zip(&update_values) {
                        point.update(index, value);
                    }
                    let elapsed = start.elapsed().as_nanos();
                    (elapsed, point.fold_lessthan(u32::MAX, 0..n) as u64)
                }),
            ),
            (
                "current_point_add",
                Box::new(|| {
                    let mut point = matrix.build_point_add::<AdditiveOperation<i64>>(&weights);
                    let start = BenchInstant::now();
                    for (&index, &value) in update_indices.iter().zip(&update_values) {
                        point.update(index, value);
                    }
                    let elapsed = start.elapsed().as_nanos();
                    (elapsed, point.fold_lessthan(u32::MAX, 0..n) as u64)
                }),
            ),
        ],
    );
}

fn bench_wavelet_existing_baseline() {
    let n = (1 << 15) + 123;
    let q = 30_000;
    let values = bench_wavelet_data(n, 1 << 13);
    let weights: Vec<i64> = values.iter().map(|&value| value as i64 % 1000).collect();
    let points: Vec<_> = values
        .iter()
        .enumerate()
        .map(|(index, &value)| (value, (index,)))
        .collect();

    bench_group(
        "wavelet_point_add_total_build",
        n,
        false,
        vec![
            (
                "wavelet_current",
                Box::new(|| {
                    let input = values.clone();
                    let input_weights = weights.clone();
                    let start = BenchInstant::now();
                    let matrix = WaveletMatrix::new(input);
                    let index = matrix.build_point_add::<AdditiveOperation<i64>>(&input_weights);
                    bench_black_box(&index);
                    let elapsed = start.elapsed().as_nanos();
                    (elapsed, index.fold_lessthan(u32::MAX, 0..n) as u64)
                }),
            ),
            (
                "compressed_bit_2d",
                Box::new(|| {
                    let input = points.clone();
                    let start = BenchInstant::now();
                    let mut index =
                        CompressedBinaryIndexedTree2d::<AdditiveOperation<i64>, u32, usize>::new(
                            &input,
                        );
                    for (point, weight) in input.iter().zip(&weights) {
                        index.update(point, weight);
                    }
                    bench_black_box(&index);
                    let elapsed = start.elapsed().as_nanos();
                    (elapsed, index.accumulate(&(.., (..,))) as u64)
                }),
            ),
        ],
    );

    let matrix = WaveletMatrix::new(values.clone());
    let point_add = matrix.build_point_add::<AdditiveOperation<i64>>(&weights);
    let mut compressed =
        CompressedBinaryIndexedTree2d::<AdditiveOperation<i64>, u32, usize>::new(&points);
    for (point, weight) in points.iter().zip(&weights) {
        compressed.update(point, weight);
    }
    let ranges = bench_wavelet_ranges(n, q, BENCH_SEED ^ 0x15_000);
    let mut rng = BenchRng::new(BENCH_SEED ^ 0x16_000);
    let thresholds: Vec<_> = (0..q).map(|_| rng.next() as u32 % (1 << 13)).collect();
    bench_group(
        "wavelet_point_add_existing_fold_lessthan",
        q,
        true,
        vec![
            (
                "wavelet_current",
                Box::new(|| {
                    bench_timed(|| {
                        thresholds
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                bench_mix(sum, point_add.fold_lessthan(value, range.clone()) as u64)
                            })
                    })
                }),
            ),
            (
                "compressed_bit_2d",
                Box::new(|| {
                    bench_timed(|| {
                        thresholds
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                let upper = compressed.accumulate(&(..value, (..range.end,)));
                                let lower = compressed.accumulate(&(..value, (..range.start,)));
                                bench_mix(sum, upper.wrapping_sub(lower) as u64)
                            })
                    })
                }),
            ),
        ],
    );
    bench_group(
        "wavelet_point_add_existing_fold_range",
        q,
        true,
        vec![
            (
                "wavelet_current",
                Box::new(|| {
                    bench_timed(|| {
                        thresholds
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                bench_mix(
                                    sum,
                                    point_add
                                        .fold_range(value..value.saturating_add(256), range.clone())
                                        as u64,
                                )
                            })
                    })
                }),
            ),
            (
                "compressed_bit_2d",
                Box::new(|| {
                    bench_timed(|| {
                        thresholds
                            .iter()
                            .zip(&ranges)
                            .fold(0, |sum, (&value, range)| {
                                let value_end = value.saturating_add(256);
                                let upper_right =
                                    compressed.accumulate(&(..value_end, (..range.end,)));
                                let upper_left =
                                    compressed.accumulate(&(..value_end, (..range.start,)));
                                let lower_right = compressed.accumulate(&(..value, (..range.end,)));
                                let lower_left =
                                    compressed.accumulate(&(..value, (..range.start,)));
                                bench_mix(
                                    sum,
                                    upper_right
                                        .wrapping_sub(upper_left)
                                        .wrapping_sub(lower_right)
                                        .wrapping_add(lower_left)
                                        as u64,
                                )
                            })
                    })
                }),
            ),
        ],
    );

    let update_indices: Vec<_> = (0..q).map(|_| rng.index(n)).collect();
    let update_values: Vec<_> = (0..q).map(|_| (rng.next() % 100) as i64).collect();
    bench_group(
        "wavelet_point_add_existing_update",
        q,
        true,
        vec![
            (
                "wavelet_current",
                Box::new(|| {
                    let mut index = matrix.build_point_add::<AdditiveOperation<i64>>(&weights);
                    let start = BenchInstant::now();
                    for (&position, &value) in update_indices.iter().zip(&update_values) {
                        index.update(position, value);
                    }
                    let elapsed = start.elapsed().as_nanos();
                    (elapsed, index.fold_lessthan(u32::MAX, 0..n) as u64)
                }),
            ),
            (
                "compressed_bit_2d",
                Box::new(|| {
                    let mut index = compressed.clone();
                    let start = BenchInstant::now();
                    for (&position, &value) in update_indices.iter().zip(&update_values) {
                        index.update(&points[position], &value);
                    }
                    let elapsed = start.elapsed().as_nanos();
                    (elapsed, index.accumulate(&(.., (..,))) as u64)
                }),
            ),
        ],
    );
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct BenchKey32(i32);

impl SimdKey for BenchKey32 {
    const BITS: u32 = 32;

    #[inline(always)]
    fn encode(self) -> u128 {
        (self.0 as u32 ^ (1 << 31)) as u128
    }
}

fn bench_static_type<K>(type_name: &str, n: usize, q: usize, make: fn(u64, u64) -> K)
where
    K: SimdKey + std::fmt::Debug + 'static,
{
    let mut rng = BenchRng::new(BENCH_SEED ^ K::BITS as u64 ^ n as u64);
    let mut values: Vec<K> = (0..n)
        .map(|_| make(rng.next() % (n as u64 / 2 + 257), rng.next()))
        .collect();
    values.sort_unstable();
    let queries: Vec<K> = (0..q)
        .map(|_| make(rng.next() % (n as u64 / 2 + 513), rng.next()))
        .collect();
    let build_checksum = bench_mix(
        values.len() as u64,
        values.partition_point(|&value| value < values[n / 2]) as u64,
    );

    let label = format!("static_{type_name}_build");
    let mut build_cases: Vec<BenchCase<'_>> = vec![(
        "slice_clone",
        Box::new(|| {
            let input = values.clone();
            let start = BenchInstant::now();
            let mut cloned = input.clone();
            bench_black_box(cloned.as_mut_slice());
            let elapsed = start.elapsed().as_nanos();
            (elapsed, build_checksum)
        }),
    )];
    for backend in bench_backends() {
        let name = bench_backend_name(backend);
        let build_values = values.clone();
        build_cases.push((
            name,
            Box::new(move || {
                let input = build_values.clone();
                let start = BenchInstant::now();
                let search = StaticSearch::benchmark_from_sorted_with_backend(&input, backend);
                bench_black_box(&search);
                let elapsed = start.elapsed().as_nanos();
                (elapsed, build_checksum)
            }),
        ));
    }
    if K::BITS == 16 {
        build_cases.push((
            "direct",
            Box::new(|| {
                let input = values.clone();
                let start = BenchInstant::now();
                let search = StaticSearch::from_sorted_direct(&input);
                bench_black_box(&search);
                let elapsed = start.elapsed().as_nanos();
                (elapsed, build_checksum)
            }),
        ));
    }
    bench_group(&label, n, false, build_cases);

    let searches: Vec<(&'static str, StaticSearch<K>)> = if K::BITS == 8 {
        vec![("direct", StaticSearch::from_sorted(&values))]
    } else {
        let mut searches: Vec<_> = bench_backends()
            .into_iter()
            .map(|backend| {
                (
                    bench_backend_name(backend),
                    StaticSearch::benchmark_from_sorted_with_backend(&values, backend),
                )
            })
            .collect();
        if K::BITS == 16 {
            searches.push(("direct", StaticSearch::from_sorted_direct(&values)));
        }
        searches
    };

    let label = format!("static_{type_name}_position");
    let mut cases: Vec<BenchCase<'_>> = vec![(
        "slice",
        Box::new(|| {
            bench_timed(|| {
                queries.iter().fold(0, |sum, &query| {
                    bench_mix(sum, values.partition_point(|&value| value < query) as u64)
                })
            })
        }),
    )];
    for &(name, ref search) in &searches {
        let queries = &queries;
        let search = search;
        cases.push((
            name,
            Box::new(move || {
                bench_timed(|| {
                    queries.iter().fold(0, |sum, &query| {
                        bench_mix(sum, search.position_bisect(query) as u64)
                    })
                })
            }),
        ));
    }
    bench_group(&label, q, true, cases);

    let label = format!("static_{type_name}_rposition");
    let mut cases: Vec<BenchCase<'_>> = vec![(
        "slice",
        Box::new(|| {
            bench_timed(|| {
                queries.iter().fold(0, |sum, &query| {
                    bench_mix(sum, values.partition_point(|&value| value <= query) as u64)
                })
            })
        }),
    )];
    for &(name, ref search) in &searches {
        let queries = &queries;
        let search = search;
        cases.push((
            name,
            Box::new(move || {
                bench_timed(|| {
                    queries.iter().fold(0, |sum, &query| {
                        bench_mix(sum, search.rposition_bisect(query) as u64)
                    })
                })
            }),
        ));
    }
    bench_group(&label, q, true, cases);

    let label = format!("static_{type_name}_range");
    let mut cases: Vec<BenchCase<'_>> = vec![(
        "slice",
        Box::new(|| {
            bench_timed(|| {
                queries.iter().fold(0, |sum, &query| {
                    let left = values.partition_point(|&value| value < query);
                    let right = values.partition_point(|&value| value <= query);
                    bench_mix(sum, (left as u64).rotate_left(29) ^ right as u64)
                })
            })
        }),
    )];
    for &(name, ref search) in &searches {
        let queries = &queries;
        let search = search;
        cases.push((
            name,
            Box::new(move || {
                bench_timed(|| {
                    queries.iter().fold(0, |sum, &query| {
                        let range = search.range(query);
                        bench_mix(sum, (range.start as u64).rotate_left(29) ^ range.end as u64)
                    })
                })
            }),
        ));
    }
    bench_group(&label, q, true, cases);

    let label = format!("static_{type_name}_contains");
    let mut cases: Vec<BenchCase<'_>> = vec![(
        "slice",
        Box::new(|| {
            bench_timed(|| {
                queries.iter().fold(0, |sum, &query| {
                    bench_mix(sum, values.binary_search(&query).is_ok() as u64)
                })
            })
        }),
    )];
    for &(name, ref search) in &searches {
        let queries = &queries;
        let search = search;
        cases.push((
            name,
            Box::new(move || {
                bench_timed(|| {
                    queries.iter().fold(0, |sum, &query| {
                        bench_mix(sum, search.contains(query) as u64)
                    })
                })
            }),
        ));
    }
    bench_group(&label, q, true, cases);

    for reverse in [false, true] {
        let label = format!(
            "static_{type_name}_{}_batch",
            if reverse { "rposition" } else { "position" }
        );
        let mut cases: Vec<BenchCase<'_>> = vec![(
            "slice",
            Box::new(|| {
                bench_timed(|| {
                    queries.iter().fold(0, |sum, &query| {
                        let position = if reverse {
                            values.partition_point(|&value| value <= query)
                        } else {
                            values.partition_point(|&value| value < query)
                        };
                        bench_mix(sum, position as u64)
                    })
                })
            }),
        )];
        for &(name, ref search) in &searches {
            let queries = &queries;
            let search = search;
            let mut output = vec![0; queries.len()];
            cases.push((
                name,
                Box::new(move || {
                    let start = BenchInstant::now();
                    if reverse {
                        search.rposition_bisect_batch(&queries, &mut output);
                    } else {
                        search.position_bisect_batch(&queries, &mut output);
                    }
                    let elapsed = start.elapsed().as_nanos();
                    let checksum = output
                        .iter()
                        .fold(0, |sum, &value| bench_mix(sum, value as u64));
                    (elapsed, checksum)
                }),
            ));
        }
        bench_group(&label, q, true, cases);
    }
}

fn bench_static_small() {
    let n = (1 << 18) + 123;
    let q = 60_000;
    bench_static_type("u8", n, q, |low, _| low as u8);
    bench_static_type("i8", n, q, |low, _| low as i8);
    bench_static_type("u16", n, q, |low, _| low as u16);
    bench_static_type("i16", n, q, |low, _| low as i16);
}

fn bench_static_medium() {
    let n = (1 << 18) + 123;
    let q = 60_000;
    bench_static_type("u32", n, q, |low, high| (low ^ high.rotate_left(17)) as u32);
    bench_static_type("i32", n, q, |low, high| (low ^ high.rotate_left(17)) as i32);
    bench_static_type("u64", n, q, |low, high| low ^ high.rotate_left(17));
    bench_static_type("i64", n, q, |low, high| (low ^ high.rotate_left(17)) as i64);
}

fn bench_static_large() {
    let n = (1 << 18) + 123;
    let q = 50_000;
    bench_static_type("u128", n, q, |low, high| (high as u128) << 64 | low as u128);
    bench_static_type("i128", n, q, |low, high| {
        ((high as u128) << 64 | low as u128) as i128
    });
    bench_static_type("usize", n, q, |low, high| {
        (low ^ high.rotate_left(17)) as usize
    });
    bench_static_type("isize", n, q, |low, high| {
        (low ^ high.rotate_left(17)) as isize
    });
    bench_static_type("custom32", n, q, |low, high| {
        BenchKey32((low ^ high.rotate_left(17)) as i32)
    });
}

macro_rules! bench_bucket_type {
    ($name:literal, $queue:ty, $value:ty, $n:expr, $q:expr, $make:expr) => {{
        let n = $n;
        let q = $q;
        let mut rng = BenchRng::new(BENCH_SEED ^ n as u64 ^ <$value>::BITS as u64);
        let values: Vec<$value> = (0..n).map(|_| ($make)(rng.next())).collect();
        let updates: Vec<$value> = (0..q).map(|_| ($make)(rng.next())).collect();
        let label = format!("bucket_{}_build", $name);
        bench_group(
            &label,
            n,
            true,
            vec![
                (
                    "binary_heap",
                    Box::new(|| {
                        let input = values.clone();
                        let start = BenchInstant::now();
                        let queue = BenchBinaryHeap::from(input);
                        bench_black_box(&queue);
                        let elapsed = start.elapsed().as_nanos();
                        (
                            elapsed,
                            bench_mix(
                                queue.len() as u64,
                                queue.peek().copied().unwrap_or(0) as u64,
                            ),
                        )
                    }),
                ),
                (
                    "bucket",
                    Box::new(|| {
                        let input = values.clone();
                        let start = BenchInstant::now();
                        let queue = <$queue>::from(input);
                        bench_black_box(&queue);
                        let elapsed = start.elapsed().as_nanos();
                        (
                            elapsed,
                            bench_mix(queue.len() as u64, queue.peek().unwrap_or(0) as u64),
                        )
                    }),
                ),
            ],
        );
        let label = format!("bucket_{}_push_only", $name);
        bench_group(
            &label,
            n,
            true,
            vec![
                (
                    "binary_heap",
                    Box::new(|| {
                        let input = values.clone();
                        let mut queue = BenchBinaryHeap::with_capacity(n);
                        let start = BenchInstant::now();
                        for value in input {
                            queue.push(value);
                        }
                        bench_black_box(&queue);
                        let elapsed = start.elapsed().as_nanos();
                        (
                            elapsed,
                            bench_mix(
                                queue.len() as u64,
                                queue.peek().copied().unwrap_or(0) as u64,
                            ),
                        )
                    }),
                ),
                (
                    "bucket",
                    Box::new(|| {
                        let input = values.clone();
                        let mut queue = <$queue>::new();
                        let start = BenchInstant::now();
                        for value in input {
                            queue.push(value);
                        }
                        bench_black_box(&queue);
                        let elapsed = start.elapsed().as_nanos();
                        (
                            elapsed,
                            bench_mix(queue.len() as u64, queue.peek().unwrap_or(0) as u64),
                        )
                    }),
                ),
            ],
        );
        let label = format!("bucket_{}_pop_drain", $name);
        bench_group(
            &label,
            n,
            true,
            vec![
                (
                    "binary_heap",
                    Box::new(|| {
                        let mut queue = BenchBinaryHeap::from(values.clone());
                        bench_timed(|| {
                            let mut checksum = 0;
                            while let Some(value) = queue.pop() {
                                checksum = bench_mix(checksum, value as u64);
                            }
                            checksum
                        })
                    }),
                ),
                (
                    "bucket",
                    Box::new(|| {
                        let mut queue = <$queue>::from(values.clone());
                        bench_timed(|| {
                            let mut checksum = 0;
                            while let Some(value) = queue.pop() {
                                checksum = bench_mix(checksum, value as u64);
                            }
                            checksum
                        })
                    }),
                ),
            ],
        );
        let label = format!("bucket_{}_pop_push", $name);
        bench_group(
            &label,
            q,
            true,
            vec![
                (
                    "binary_heap",
                    Box::new(|| {
                        let mut queue = BenchBinaryHeap::from(values.clone());
                        bench_timed(|| {
                            updates.iter().fold(0, |checksum, &value| {
                                let removed = queue.pop().unwrap();
                                queue.push(value);
                                bench_mix(checksum, removed as u64)
                            })
                        })
                    }),
                ),
                (
                    "bucket",
                    Box::new(|| {
                        let mut queue = <$queue>::from(values.clone());
                        bench_timed(|| {
                            updates.iter().fold(0, |checksum, &value| {
                                let removed = queue.pop().unwrap();
                                queue.push(value);
                                bench_mix(checksum, removed as u64)
                            })
                        })
                    }),
                ),
            ],
        );
        let label = format!("bucket_{}_replace_random", $name);
        bench_group(
            &label,
            q,
            true,
            vec![
                (
                    "binary_heap",
                    Box::new(|| {
                        let mut queue = BenchBinaryHeap::from(values.clone());
                        bench_timed(|| {
                            updates.iter().fold(0, |checksum, &value| {
                                let removed = queue.pop().unwrap();
                                queue.push(value);
                                bench_mix(checksum, removed as u64)
                            })
                        })
                    }),
                ),
                (
                    "bucket",
                    Box::new(|| {
                        let mut queue = <$queue>::from(values.clone());
                        bench_timed(|| {
                            updates.iter().fold(0, |checksum, &value| {
                                bench_mix(checksum, queue.replace(value).unwrap() as u64)
                            })
                        })
                    }),
                ),
            ],
        );
        let label = format!("bucket_{}_replace_same_max", $name);
        bench_group(
            &label,
            q,
            true,
            vec![
                (
                    "binary_heap",
                    Box::new(|| {
                        let mut queue = BenchBinaryHeap::from(values.clone());
                        bench_timed(|| {
                            (0..q).fold(0, |checksum, _| {
                                let value = queue.pop().unwrap();
                                queue.push(value);
                                bench_mix(checksum, value as u64)
                            })
                        })
                    }),
                ),
                (
                    "bucket",
                    Box::new(|| {
                        let mut queue = <$queue>::from(values.clone());
                        bench_timed(|| {
                            (0..q).fold(0, |checksum, _| {
                                let value = queue.peek().unwrap();
                                bench_mix(checksum, queue.replace(value).unwrap() as u64)
                            })
                        })
                    }),
                ),
            ],
        );
        let repetitions = 32;
        let label = format!("bucket_{}_clear", $name);
        bench_group(
            &label,
            repetitions,
            true,
            vec![
                (
                    "binary_heap",
                    Box::new(|| {
                        let base = BenchBinaryHeap::from(values.clone());
                        let mut elapsed = 0;
                        let mut checksum = 0;
                        for _ in 0..repetitions {
                            let mut queue = base.clone();
                            let start = BenchInstant::now();
                            queue.clear();
                            elapsed += start.elapsed().as_nanos();
                            checksum = bench_mix(checksum, queue.len() as u64);
                        }
                        (elapsed, checksum)
                    }),
                ),
                (
                    "bucket",
                    Box::new(|| {
                        let base = <$queue>::from(values.clone());
                        let mut elapsed = 0;
                        let mut checksum = 0;
                        for _ in 0..repetitions {
                            let mut queue = base.clone();
                            let start = BenchInstant::now();
                            queue.clear();
                            elapsed += start.elapsed().as_nanos();
                            checksum = bench_mix(checksum, queue.len() as u64);
                        }
                        (elapsed, checksum)
                    }),
                ),
            ],
        );
    }};
}

fn bench_bucket_queue() {
    bench_bucket_type!(
        "u8",
        BucketQueueU8,
        u8,
        1 << 16,
        120_000,
        |value: u64| value as u8
    );
    bench_bucket_type!(
        "i8",
        BucketQueueI8,
        i8,
        1 << 16,
        120_000,
        |value: u64| value as i8
    );
    bench_bucket_type!(
        "u16",
        BucketQueueU16,
        u16,
        1 << 18,
        120_000,
        |value: u64| value as u16
    );
    bench_bucket_type!(
        "i16",
        BucketQueueI16,
        i16,
        1 << 18,
        120_000,
        |value: u64| value as i16
    );
}

macro_rules! bench_dary_type {
    ($name:expr, $heap:ty, $value:ty, $n:expr, $q:expr, $make:expr) => {{
        let n = $n;
        let q = $q;
        let mut rng = BenchRng::new(BENCH_SEED ^ n as u64 ^ <$value>::BITS as u64 ^ $name.len() as u64);
        let values: Vec<$value> = (0..n).map(|_| ($make)(&mut rng)).collect();
        let updates: Vec<$value> = (0..q).map(|_| ($make)(&mut rng)).collect();
        let label = format!("dary_{}_n{}_build", $name, n);
        let mut cases: Vec<BenchCase<'_>> = vec![(
            "binary_heap",
            Box::new(|| {
                let input = values.clone();
                let start = BenchInstant::now();
                let heap = BenchBinaryHeap::from(input);
                bench_black_box(&heap);
                let elapsed = start.elapsed().as_nanos();
                (
                    elapsed,
                    bench_mix(
                        heap.len() as u64,
                        bench_integer_hash(heap.peek().copied().unwrap_or(0) as u128),
                    ),
                )
            }),
        )];
        for backend in bench_backends() {
            let name = bench_backend_name(backend);
            let input_values = values.clone();
            cases.push((
                name,
                Box::new(move || {
                    let input = input_values.clone();
                    let start = BenchInstant::now();
                    let heap = <$heap>::benchmark_build(input, backend);
                    bench_black_box(&heap);
                    let elapsed = start.elapsed().as_nanos();
                    (
                        elapsed,
                        bench_mix(
                            heap.len() as u64,
                            bench_integer_hash(heap.peek().unwrap_or(0) as u128),
                        ),
                    )
                }),
            ));
        }
        bench_group(&label, n, true, cases);

        let label = format!("dary_{}_n{}_push_only", $name, n);
        let mut cases: Vec<BenchCase<'_>> = vec![(
            "binary_heap",
            Box::new(|| {
                let input = values.clone();
                let mut heap = BenchBinaryHeap::with_capacity(n);
                let start = BenchInstant::now();
                for value in input {
                    heap.push(value);
                }
                bench_black_box(&heap);
                let elapsed = start.elapsed().as_nanos();
                (
                    elapsed,
                    bench_mix(
                        heap.len() as u64,
                        bench_integer_hash(heap.peek().copied().unwrap_or(0) as u128),
                    ),
                )
            }),
        )];
        for backend in bench_backends() {
            let name = bench_backend_name(backend);
            let input_values = values.clone();
            cases.push((
                name,
                Box::new(move || {
                    let input = input_values.clone();
                    let mut heap = <$heap>::benchmark_empty(n, backend);
                    let start = BenchInstant::now();
                    for value in input {
                        heap.push(value);
                    }
                    bench_black_box(&heap);
                    let elapsed = start.elapsed().as_nanos();
                    (
                        elapsed,
                        bench_mix(
                            heap.len() as u64,
                            bench_integer_hash(heap.peek().unwrap_or(0) as u128),
                        ),
                    )
                }),
            ));
        }
        bench_group(&label, n, true, cases);

        let label = format!("dary_{}_n{}_pop_drain", $name, n);
        let mut cases: Vec<BenchCase<'_>> = vec![(
            "binary_heap",
            Box::new(|| {
                let mut heap = BenchBinaryHeap::from(values.clone());
                bench_timed(|| {
                    let mut checksum = 0;
                    while let Some(value) = heap.pop() {
                        checksum = bench_mix(checksum, bench_integer_hash(value as u128));
                    }
                    checksum
                })
            }),
        )];
        for backend in bench_backends() {
            let name = bench_backend_name(backend);
            let input_values = values.clone();
            cases.push((
                name,
                Box::new(move || {
                    let mut heap = <$heap>::benchmark_build(input_values.clone(), backend);
                    bench_timed(|| {
                        let mut checksum = 0;
                        while let Some(value) = heap.pop() {
                            checksum = bench_mix(checksum, bench_integer_hash(value as u128));
                        }
                        checksum
                    })
                }),
            ));
        }
        bench_group(&label, n, true, cases);

        let label = format!("dary_{}_n{}_into_sorted_vec", $name, n);
        let mut cases: Vec<BenchCase<'_>> = vec![(
            "binary_heap",
            Box::new(|| {
                let heap = BenchBinaryHeap::from(values.clone());
                let start = BenchInstant::now();
                let sorted = heap.into_sorted_vec();
                let elapsed = start.elapsed().as_nanos();
                let checksum = sorted.iter().fold(0, |sum, &value| {
                    bench_mix(sum, bench_integer_hash(value as u128))
                });
                (elapsed, checksum)
            }),
        )];
        for backend in bench_backends() {
            let name = bench_backend_name(backend);
            let input_values = values.clone();
            cases.push((
                name,
                Box::new(move || {
                    let heap = <$heap>::benchmark_build(input_values.clone(), backend);
                    let start = BenchInstant::now();
                    let sorted = heap.into_sorted_vec();
                    let elapsed = start.elapsed().as_nanos();
                    let checksum = sorted.iter().fold(0, |sum, &value| {
                        bench_mix(sum, bench_integer_hash(value as u128))
                    });
                    (elapsed, checksum)
                }),
            ));
        }
        bench_group(&label, n, true, cases);

        for operation in ["pop_push", "replace", "top_k", "replace_same_max"] {
            if operation == "top_k" {
                let mut heap = BenchBinaryHeap::from(values.clone());
                let mut replacements = 0;
                for &value in &updates {
                    if value < *heap.peek().unwrap() {
                        heap.pop();
                        heap.push(value);
                        replacements += 1;
                    }
                }
                println!(
                    "workload=dary_{}_n{}_top_k operations={} replacements={} skips={} replacement_rate={:.6}",
                    $name,
                    n,
                    q,
                    replacements,
                    q - replacements,
                    replacements as f64 / q as f64,
                );
            }
            let label = format!("dary_{}_n{}_{}", $name, n, operation);
            let mut cases: Vec<BenchCase<'_>> = vec![(
                "binary_heap",
                Box::new(|| {
                    let mut heap = BenchBinaryHeap::from(values.clone());
                    bench_timed(|| {
                        if operation == "replace_same_max" {
                            return (0..q).fold(0, |checksum, _| {
                                let value = heap.pop().unwrap();
                                heap.push(value);
                                bench_mix(checksum, bench_integer_hash(value as u128))
                            });
                        }
                        updates.iter().fold(0, |checksum, &value| {
                            if operation == "top_k" && value >= *heap.peek().unwrap() {
                                bench_mix(checksum, 0)
                            } else {
                                let removed = heap.pop().unwrap();
                                heap.push(value);
                                bench_mix(checksum, bench_integer_hash(removed as u128))
                            }
                        })
                    })
                }),
            )];
            for backend in bench_backends() {
                let name = bench_backend_name(backend);
                let input_values = values.clone();
                let input_updates = updates.clone();
                cases.push((
                    name,
                    Box::new(move || {
                        let mut heap = <$heap>::benchmark_build(input_values.clone(), backend);
                        bench_timed(|| {
                            if operation == "replace_same_max" {
                                return (0..q).fold(0, |checksum, _| {
                                    let value = heap.peek().unwrap();
                                    bench_mix(
                                        checksum,
                                        bench_integer_hash(heap.replace(value).unwrap() as u128),
                                    )
                                });
                            }
                            input_updates.iter().fold(0, |checksum, &value| {
                                if operation == "top_k" && value >= heap.peek().unwrap() {
                                    bench_mix(checksum, 0)
                                } else {
                                    let removed = if operation == "pop_push" {
                                        let removed = heap.pop().unwrap();
                                        heap.push(value);
                                        removed
                                    } else {
                                        heap.replace(value).unwrap()
                                    };
                                    bench_mix(checksum, bench_integer_hash(removed as u128))
                                }
                            })
                        })
                    }),
                ));
            }
            bench_group(&label, q, true, cases);
        }
        if n == 65_536 {
            let repetitions = 32;
            let label = format!("dary_{}_n{}_clear", $name, n);
            let mut cases: Vec<BenchCase<'_>> = vec![(
                "binary_heap",
                Box::new(|| {
                    let base = BenchBinaryHeap::from(values.clone());
                    let mut elapsed = 0;
                    let mut checksum = 0;
                    for _ in 0..repetitions {
                        let mut heap = base.clone();
                        let start = BenchInstant::now();
                        heap.clear();
                        elapsed += start.elapsed().as_nanos();
                        checksum = bench_mix(checksum, heap.len() as u64);
                    }
                    (elapsed, checksum)
                }),
            )];
            for backend in bench_backends() {
                let name = bench_backend_name(backend);
                let input_values = values.clone();
                cases.push((
                    name,
                    Box::new(move || {
                        let base = <$heap>::benchmark_build(input_values.clone(), backend);
                        let mut elapsed = 0;
                        let mut checksum = 0;
                        for _ in 0..repetitions {
                            let mut heap = base.clone();
                            let start = BenchInstant::now();
                            heap.clear();
                            elapsed += start.elapsed().as_nanos();
                            checksum = bench_mix(checksum, heap.len() as u64);
                        }
                        (elapsed, checksum)
                    }),
                ));
            }
            bench_group(&label, repetitions, true, cases);
        }
    }};
}

fn bench_dary_u32_small() {
    for n in [4_096, 65_536] {
        bench_dary_type!(
            "u32_random",
            DaryHeapU32,
            u32,
            n,
            60_000,
            |rng: &mut BenchRng| rng.next() as u32
        );
    }
}

fn bench_dary_u32_large() {
    bench_dary_type!(
        "u32_random",
        DaryHeapU32,
        u32,
        1 << 20,
        60_000,
        |rng: &mut BenchRng| rng.next() as u32
    );
}

fn bench_dary_u32_variants() {
    bench_dary_type!(
        "u32_repeat64",
        DaryHeapU32,
        u32,
        1 << 20,
        60_000,
        |rng: &mut BenchRng| (rng.next() % 64) as u32
    );
    bench_dary_type!(
        "i32_random",
        DaryHeapI32,
        i32,
        65_536,
        80_000,
        |rng: &mut BenchRng| rng.next() as i32
    );
}

fn bench_dary_u64_small() {
    for n in [4_096, 65_536] {
        bench_dary_type!(
            "u64_random",
            DaryHeapU64,
            u64,
            n,
            60_000,
            |rng: &mut BenchRng| rng.next()
        );
    }
}

fn bench_dary_u64_large() {
    bench_dary_type!(
        "u64_random",
        DaryHeapU64,
        u64,
        1 << 20,
        60_000,
        |rng: &mut BenchRng| rng.next()
    );
}

fn bench_dary_u64_variants() {
    bench_dary_type!(
        "u64_repeat64",
        DaryHeapU64,
        u64,
        1 << 20,
        60_000,
        |rng: &mut BenchRng| rng.next() % 64
    );
    bench_dary_type!(
        "i64_random",
        DaryHeapI64,
        i64,
        65_536,
        80_000,
        |rng: &mut BenchRng| rng.next() as i64
    );
}

fn bench_dary_u128_small() {
    for n in [4_096, 65_536] {
        bench_dary_type!(
            "u128_random",
            DaryHeapU128,
            u128,
            n,
            40_000,
            |rng: &mut BenchRng| { (rng.next() as u128) << 64 | rng.next() as u128 }
        );
    }
}

fn bench_dary_u128_large() {
    bench_dary_type!(
        "u128_random",
        DaryHeapU128,
        u128,
        1 << 20,
        40_000,
        |rng: &mut BenchRng| { (rng.next() as u128) << 64 | rng.next() as u128 }
    );
}

fn bench_dary_u128_variants() {
    bench_dary_type!(
        "u128_repeat64",
        DaryHeapU128,
        u128,
        1 << 20,
        40_000,
        |rng: &mut BenchRng| rng.next() as u128 % 64
    );
    bench_dary_type!(
        "i128_random",
        DaryHeapI128,
        i128,
        65_536,
        60_000,
        |rng: &mut BenchRng| { ((rng.next() as u128) << 64 | rng.next() as u128) as i128 }
    );
}

trait BenchUnsigned: Copy + Default + Ord {
    fn wrapping_add(self, other: Self) -> Self;
    fn wrapping_sub(self, other: Self) -> Self;
}

impl BenchUnsigned for u32 {
    fn wrapping_add(self, other: Self) -> Self {
        self.wrapping_add(other)
    }

    fn wrapping_sub(self, other: Self) -> Self {
        self.wrapping_sub(other)
    }
}

impl BenchUnsigned for u64 {
    fn wrapping_add(self, other: Self) -> Self {
        self.wrapping_add(other)
    }

    fn wrapping_sub(self, other: Self) -> Self {
        self.wrapping_sub(other)
    }
}

#[derive(Clone)]
struct BenchFenwick<T: BenchUnsigned> {
    bit: Vec<T>,
}

impl<T: BenchUnsigned> BenchFenwick<T> {
    fn zeroed(len: usize) -> Self {
        Self {
            bit: vec![T::default(); len + 1],
        }
    }

    fn from_slice(values: &[T]) -> Self {
        let mut bit = vec![T::default(); values.len() + 1];
        for (index, &value) in values.iter().enumerate() {
            let position = index + 1;
            bit[position] = bit[position].wrapping_add(value);
            let parent = position + (position & position.wrapping_neg());
            if parent <= values.len() {
                bit[parent] = bit[parent].wrapping_add(bit[position]);
            }
        }
        Self { bit }
    }

    fn len(&self) -> usize {
        self.bit.len() - 1
    }

    #[inline]
    fn update(&mut self, index: usize, value: T) {
        let mut position = index + 1;
        while position < self.bit.len() {
            self.bit[position] = self.bit[position].wrapping_add(value);
            position += position & position.wrapping_neg();
        }
    }

    #[inline]
    fn accumulate0(&self, mut end: usize) -> T {
        let mut result = T::default();
        while end != 0 {
            result = result.wrapping_add(self.bit[end]);
            end -= end & end.wrapping_neg();
        }
        result
    }

    #[inline]
    fn accumulate(&self, index: usize) -> T {
        self.accumulate0(index + 1)
    }

    #[inline]
    fn fold(&self, left: usize, right: usize) -> T {
        self.accumulate0(right).wrapping_sub(self.accumulate0(left))
    }

    #[inline]
    fn get(&self, index: usize) -> T {
        self.fold(index, index + 1)
    }

    #[inline]
    fn set(&mut self, index: usize, value: T) {
        self.update(index, value.wrapping_sub(self.get(index)));
    }

    #[inline]
    fn partition_point_acc(&self, target: T) -> usize {
        let mut position = 0;
        let mut sum = T::default();
        let mut step = self.len().next_power_of_two();
        while step != 0 {
            if position + step <= self.len() {
                let next = sum.wrapping_add(self.bit[position + step]);
                if next <= target {
                    position += step;
                    sum = next;
                }
            }
            step >>= 1;
        }
        position
    }
}

macro_rules! bench_wide_prefix_type {
    ($name:literal, $wide:ty, $value:ty, $n:expr, $q:expr) => {{
        let n = $n;
        let q = $q;
        let mut rng = BenchRng::new(BENCH_SEED ^ <$value>::BITS as u64 ^ 0x7000);
        let values: Vec<$value> = (0..n).map(|_| (rng.next() % 8) as $value).collect();
        let indices: Vec<_> = (0..q).map(|_| rng.index(n)).collect();
        let deltas: Vec<$value> = (0..q).map(|_| (rng.next() % 8) as $value).collect();
        let replacements: Vec<$value> = (0..q).map(|_| (rng.next() % 64) as $value).collect();
        let ends: Vec<_> = (0..q).map(|_| rng.index(n + 1)).collect();
        let ranges: Vec<_> = (0..q)
            .map(|_| {
                let left = rng.index(n);
                let right = left + rng.index((n - left).min(4096) + 1);
                (left, right)
            })
            .collect();
        let total: $value = values.iter().copied().sum();
        let targets: Vec<$value> = (0..q)
            .map(|_| {
                if total == 0 {
                    0
                } else {
                    (rng.next() as $value) % total
                }
            })
            .collect();

        let label = format!("wide_prefix_{}_new", $name);
        let mut cases: Vec<BenchCase<'_>> = vec![(
            "binary_indexed_tree",
            Box::new(|| {
                let start = BenchInstant::now();
                let tree = BenchFenwick::<$value>::zeroed(n);
                bench_black_box(&tree);
                let elapsed = start.elapsed().as_nanos();
                (elapsed, tree.accumulate0(n) as u64)
            }),
        )];
        for backend in bench_backends() {
            let name = bench_backend_name(backend);
            cases.push((
                name,
                Box::new(move || {
                    let start = BenchInstant::now();
                    let tree = <$wide>::benchmark_zeroed(n, backend);
                    bench_black_box(&tree);
                    let elapsed = start.elapsed().as_nanos();
                    (elapsed, tree.fold_all() as u64)
                }),
            ));
        }
        bench_group(&label, n, true, cases);

        let label = format!("wide_prefix_{}_build", $name);
        let mut cases: Vec<BenchCase<'_>> = vec![(
            "binary_indexed_tree",
            Box::new(|| {
                let input = values.clone();
                let start = BenchInstant::now();
                let tree = BenchFenwick::<$value>::from_slice(&input);
                bench_black_box(&tree);
                let elapsed = start.elapsed().as_nanos();
                (elapsed, tree.accumulate0(n) as u64)
            }),
        )];
        for backend in bench_backends() {
            let name = bench_backend_name(backend);
            let input_values = values.clone();
            cases.push((
                name,
                Box::new(move || {
                    let input = input_values.clone();
                    let start = BenchInstant::now();
                    let tree = <$wide>::benchmark_build(&input, backend);
                    bench_black_box(&tree);
                    let elapsed = start.elapsed().as_nanos();
                    (elapsed, tree.fold_all() as u64)
                }),
            ));
        }
        bench_group(&label, n, true, cases);

        let label = format!("wide_prefix_{}_update", $name);
        let mut cases: Vec<BenchCase<'_>> = vec![(
            "binary_indexed_tree",
            Box::new(|| {
                let mut tree = BenchFenwick::<$value>::from_slice(&values);
                let start = BenchInstant::now();
                for (&index, &value) in indices.iter().zip(&deltas) {
                    tree.update(index, value);
                }
                let elapsed = start.elapsed().as_nanos();
                (elapsed, tree.accumulate0(n) as u64)
            }),
        )];
        for backend in bench_backends() {
            let name = bench_backend_name(backend);
            let values = &values;
            let indices = &indices;
            let deltas = &deltas;
            cases.push((
                name,
                Box::new(move || {
                    let mut tree = <$wide>::benchmark_build(values, backend);
                    let start = BenchInstant::now();
                    for (&index, &value) in indices.iter().zip(deltas.iter()) {
                        tree.update(index, value);
                    }
                    let elapsed = start.elapsed().as_nanos();
                    (elapsed, tree.fold_all() as u64)
                }),
            ));
        }
        bench_group(&label, q, true, cases);

        let label = format!("wide_prefix_{}_set", $name);
        let mut cases: Vec<BenchCase<'_>> = vec![(
            "binary_indexed_tree",
            Box::new(|| {
                let mut tree = BenchFenwick::<$value>::from_slice(&values);
                let start = BenchInstant::now();
                for (&index, &value) in indices.iter().zip(&replacements) {
                    tree.set(index, value);
                }
                let elapsed = start.elapsed().as_nanos();
                (elapsed, tree.accumulate0(n) as u64)
            }),
        )];
        for backend in bench_backends() {
            let name = bench_backend_name(backend);
            let values = &values;
            let indices = &indices;
            let replacements = &replacements;
            cases.push((
                name,
                Box::new(move || {
                    let mut tree = <$wide>::benchmark_build(values, backend);
                    let start = BenchInstant::now();
                    for (&index, &value) in indices.iter().zip(replacements.iter()) {
                        tree.set(index, value);
                    }
                    let elapsed = start.elapsed().as_nanos();
                    (elapsed, tree.fold_all() as u64)
                }),
            ));
        }
        bench_group(&label, q, true, cases);

        for operation in [
            "accumulate0",
            "accumulate",
            "fold",
            "get",
            "fold_all",
            "partition_point",
        ] {
            let label = format!("wide_prefix_{}_{}", $name, operation);
            let mut cases: Vec<BenchCase<'_>> = vec![(
                "binary_indexed_tree",
                Box::new(|| {
                    let tree = BenchFenwick::<$value>::from_slice(&values);
                    bench_timed(|| match operation {
                        "accumulate0" => ends
                            .iter()
                            .fold(0, |sum, &end| bench_mix(sum, tree.accumulate0(end) as u64)),
                        "accumulate" => indices.iter().fold(0, |sum, &index| {
                            bench_mix(sum, tree.accumulate(index) as u64)
                        }),
                        "fold" => ranges.iter().fold(0, |sum, &(left, right)| {
                            bench_mix(sum, tree.fold(left, right) as u64)
                        }),
                        "get" => indices
                            .iter()
                            .fold(0, |sum, &index| bench_mix(sum, tree.get(index) as u64)),
                        "fold_all" => {
                            (0..q).fold(0, |sum, _| bench_mix(sum, tree.accumulate0(n) as u64))
                        }
                        "partition_point" => targets.iter().fold(0, |sum, &target| {
                            bench_mix(sum, tree.partition_point_acc(target) as u64)
                        }),
                        _ => unreachable!(),
                    })
                }),
            )];
            for backend in bench_backends() {
                let name = bench_backend_name(backend);
                let values = &values;
                let ends = &ends;
                let indices = &indices;
                let ranges = &ranges;
                let targets = &targets;
                cases.push((
                    name,
                    Box::new(move || {
                        let tree = <$wide>::benchmark_build(values, backend);
                        bench_timed(|| match operation {
                            "accumulate0" => ends
                                .iter()
                                .fold(0, |sum, &end| bench_mix(sum, tree.accumulate0(end) as u64)),
                            "accumulate" => indices.iter().fold(0, |sum, &index| {
                                bench_mix(sum, tree.accumulate(index) as u64)
                            }),
                            "fold" => ranges.iter().fold(0, |sum, &(left, right)| {
                                bench_mix(sum, tree.fold(left, right) as u64)
                            }),
                            "get" => indices
                                .iter()
                                .fold(0, |sum, &index| bench_mix(sum, tree.get(index) as u64)),
                            "fold_all" => {
                                (0..q).fold(0, |sum, _| bench_mix(sum, tree.fold_all() as u64))
                            }
                            "partition_point" => targets.iter().fold(0, |sum, &target| {
                                bench_mix(sum, tree.partition_point_acc(target) as u64)
                            }),
                            _ => unreachable!(),
                        })
                    }),
                ));
            }
            bench_group(&label, q, true, cases);
        }
    }};
}

fn bench_wide_prefix_u32() {
    bench_wide_prefix_type!("u32", WidePrefixU32, u32, (1 << 20) + 123, 180_000);
}

fn bench_wide_prefix_u64() {
    bench_wide_prefix_type!("u64", WidePrefixU64, u64, (1 << 20) + 123, 180_000);
}

macro_rules! bench_wide_segment_type {
    ($name:literal, $wide:ty, $operation:ty, $value:ty, $unit:expr, $branch:expr, $has_rmq:expr) => {{
        let n = (1 << 20) + 123;
        let q = 140_000;
        let mut rng = BenchRng::new(BENCH_SEED ^ <$value>::BITS as u64 ^ $name.len() as u64);
        let values: Vec<$value> = (0..n)
            .map(|_| (rng.next() % 2_000_001) as $value - 1_000_000)
            .collect();
        let indices: Vec<_> = (0..q).map(|_| rng.index(n)).collect();
        let updates: Vec<$value> = (0..q)
            .map(|_| (rng.next() % 2_000_001) as $value - 1_000_000)
            .collect();
        let short_ranges: Vec<_> = (0..q)
            .map(|_| {
                let left = rng.index(n);
                let length = 1 + rng.index((n - left).min(16));
                (left, left + length)
            })
            .collect();
        let medium_ranges: Vec<_> = (0..q)
            .map(|_| {
                let left = rng.index(n);
                let length = 1 + rng.index((n - left).min(4096));
                (left, left + length)
            })
            .collect();
        let long_ranges: Vec<_> = (0..q)
            .map(|_| {
                let length = n / 4 + rng.index(n / 4);
                let left = rng.index(n - length + 1);
                (left, left + length)
            })
            .collect();
        let aligned_ranges: Vec<_> = (0..q)
            .map(|_| {
                let blocks = n / $branch;
                let left_block = rng.index(blocks);
                let length_blocks = 1 + rng.index((blocks - left_block).min(256));
                (left_block * $branch, (left_block + length_blocks) * $branch)
            })
            .collect();

        let label = format!("wide_segment_{}_new", $name);
        let mut cases: Vec<BenchCase<'_>> = vec![(
            "segment_tree",
            Box::new(|| {
                let start = BenchInstant::now();
                let tree = SegmentTree::<$operation>::new(n);
                bench_black_box(&tree);
                let elapsed = start.elapsed().as_nanos();
                (elapsed, tree.fold_all() as u64)
            }),
        )];
        for backend in bench_backends() {
            let name = bench_backend_name(backend);
            cases.push((
                name,
                Box::new(move || {
                    let start = BenchInstant::now();
                    let tree = <$wide>::benchmark_zeroed(n, backend);
                    bench_black_box(&tree);
                    let elapsed = start.elapsed().as_nanos();
                    (elapsed, tree.fold_all() as u64)
                }),
            ));
        }
        bench_group(&label, n, true, cases);

        let label = format!("wide_segment_{}_build", $name);
        let mut cases: Vec<BenchCase<'_>> = vec![(
            "segment_tree",
            Box::new(|| {
                let input = values.clone();
                let start = BenchInstant::now();
                let tree = SegmentTree::<$operation>::from_vec(input);
                bench_black_box(&tree);
                let elapsed = start.elapsed().as_nanos();
                (elapsed, tree.fold_all() as u64)
            }),
        )];
        if $has_rmq {
            cases.push((
                "range_minimum_query",
                Box::new(|| {
                    let input = values.clone();
                    let start = BenchInstant::now();
                    let tree = RangeMinimumQuery::new(input);
                    bench_black_box(&tree);
                    let elapsed = start.elapsed().as_nanos();
                    (elapsed, tree.fold(0, n) as u64)
                }),
            ));
        }
        for backend in bench_backends() {
            let name = bench_backend_name(backend);
            let input_values = values.clone();
            cases.push((
                name,
                Box::new(move || {
                    let input = input_values.clone();
                    let start = BenchInstant::now();
                    let tree = <$wide>::benchmark_build(input, backend);
                    bench_black_box(&tree);
                    let elapsed = start.elapsed().as_nanos();
                    (elapsed, tree.fold_all() as u64)
                }),
            ));
        }
        bench_group(&label, n, true, cases);

        for method in ["set_changed", "set_same", "update", "clear"] {
            let label = format!("wide_segment_{}_{}", $name, method);
            let mut cases: Vec<BenchCase<'_>> = vec![(
                "segment_tree",
                Box::new(|| {
                    let mut tree = SegmentTree::<$operation>::from_vec(values.clone());
                    let start = BenchInstant::now();
                    for (query, (&index, &value)) in indices.iter().zip(&updates).enumerate() {
                        match method {
                            "set_changed" => tree.set(index, value),
                            "set_same" => tree.set(index, values[index]),
                            "update" => tree.update(index, value),
                            "clear" => tree.clear(index),
                            _ => unreachable!(),
                        }
                        bench_black_box(query);
                    }
                    let elapsed = start.elapsed().as_nanos();
                    (elapsed, tree.fold_all() as u64)
                }),
            )];
            for backend in bench_backends() {
                let name = bench_backend_name(backend);
                let values = &values;
                let indices = &indices;
                let updates = &updates;
                cases.push((
                    name,
                    Box::new(move || {
                        let mut tree = <$wide>::benchmark_build((*values).clone(), backend);
                        let start = BenchInstant::now();
                        for (query, (&index, &value)) in indices.iter().zip(updates).enumerate() {
                            match method {
                                "set_changed" => tree.set(index, value),
                                "set_same" => tree.set(index, values[index]),
                                "update" => tree.update(index, value),
                                "clear" => tree.clear(index),
                                _ => unreachable!(),
                            }
                            bench_black_box(query);
                        }
                        let elapsed = start.elapsed().as_nanos();
                        (elapsed, tree.fold_all() as u64)
                    }),
                ));
            }
            bench_group(&label, q, true, cases);
        }

        for (range_name, ranges) in [
            ("fold_short", &short_ranges),
            ("fold_medium", &medium_ranges),
            ("fold_long", &long_ranges),
            ("fold_aligned", &aligned_ranges),
        ] {
            let label = format!("wide_segment_{}_{}", $name, range_name);
            let segment = SegmentTree::<$operation>::from_vec(values.clone());
            let mut cases: Vec<BenchCase<'_>> = vec![(
                "segment_tree",
                Box::new(|| {
                    bench_timed(|| {
                        ranges.iter().fold(0, |sum, &(left, right)| {
                            bench_mix(sum, segment.fold(left..right) as u64)
                        })
                    })
                }),
            )];
            if $has_rmq {
                let rmq = RangeMinimumQuery::new(values.clone());
                cases.push((
                    "range_minimum_query",
                    Box::new(move || {
                        bench_timed(|| {
                            ranges.iter().fold(0, |sum, &(left, right)| {
                                bench_mix(sum, rmq.fold(left, right) as u64)
                            })
                        })
                    }),
                ));
            }
            for backend in bench_backends() {
                let name = bench_backend_name(backend);
                let tree = <$wide>::benchmark_build(values.clone(), backend);
                cases.push((
                    name,
                    Box::new(move || {
                        bench_timed(|| {
                            ranges.iter().fold(0, |sum, &(left, right)| {
                                bench_mix(sum, tree.fold(left..right) as u64)
                            })
                        })
                    }),
                ));
            }
            bench_group(&label, q, true, cases);
        }

        for method in ["get", "fold_all"] {
            let label = format!("wide_segment_{}_{}", $name, method);
            let segment = SegmentTree::<$operation>::from_vec(values.clone());
            let mut cases: Vec<BenchCase<'_>> = vec![(
                "segment_tree",
                Box::new(|| {
                    bench_timed(|| {
                        indices.iter().fold(0, |sum, &index| {
                            let value = if method == "get" {
                                segment.get(index)
                            } else {
                                segment.fold_all()
                            };
                            bench_mix(sum, value as u64)
                        })
                    })
                }),
            )];
            for backend in bench_backends() {
                let name = bench_backend_name(backend);
                let tree = <$wide>::benchmark_build(values.clone(), backend);
                let indices = &indices;
                cases.push((
                    name,
                    Box::new(move || {
                        bench_timed(|| {
                            indices.iter().fold(0, |sum, &index| {
                                let value = if method == "get" {
                                    tree.get(index)
                                } else {
                                    tree.fold_all()
                                };
                                bench_mix(sum, value as u64)
                            })
                        })
                    }),
                ));
            }
            bench_group(&label, q, true, cases);
        }
    }};
}

fn bench_wide_segment_i32() {
    bench_wide_segment_type!(
        "min_i32",
        WideSegmentTreeMinI32,
        MinOperation<i32>,
        i32,
        i32::MAX,
        16,
        true
    );
    bench_wide_segment_type!(
        "max_i32",
        WideSegmentTreeMaxI32,
        MaxOperation<i32>,
        i32,
        i32::MIN,
        16,
        false
    );
}

fn bench_wide_segment_i64() {
    bench_wide_segment_type!(
        "min_i64",
        WideSegmentTreeMinI64,
        MinOperation<i64>,
        i64,
        i64::MAX,
        8,
        true
    );
    bench_wide_segment_type!(
        "max_i64",
        WideSegmentTreeMaxI64,
        MaxOperation<i64>,
        i64,
        i64::MIN,
        8,
        false
    );
}

macro_rules! define_bench_radix_heap {
    ($name:ident, $key:ty, $buckets:expr) => {
        #[derive(Clone)]
        struct $name<T> {
            buckets: [Vec<($key, T)>; $buckets],
            last: $key,
            len: usize,
        }

        impl<T> $name<T> {
            fn new() -> Self {
                Self {
                    buckets: std::array::from_fn(|_| Vec::new()),
                    last: 0,
                    len: 0,
                }
            }

            #[inline]
            fn len(&self) -> usize {
                self.len
            }

            #[inline]
            fn bucket_index(key: $key, last: $key) -> usize {
                if key == last {
                    0
                } else {
                    <$key>::BITS as usize - (key ^ last).leading_zeros() as usize
                }
            }

            #[inline]
            fn push(&mut self, key: $key, value: T) {
                assert!(key >= self.last);
                self.buckets[Self::bucket_index(key, self.last)].push((key, value));
                self.len += 1;
            }

            fn pop(&mut self) -> Option<($key, T)> {
                if self.len == 0 {
                    return None;
                }
                if self.buckets[0].is_empty() {
                    let index = (1..self.buckets.len())
                        .find(|&index| !self.buckets[index].is_empty())
                        .unwrap();
                    self.last = self.buckets[index]
                        .iter()
                        .map(|&(key, _)| key)
                        .min()
                        .unwrap();
                    let values = std::mem::take(&mut self.buckets[index]);
                    for (key, value) in values {
                        self.buckets[Self::bucket_index(key, self.last)].push((key, value));
                    }
                }
                self.len -= 1;
                self.buckets[0].pop()
            }
        }
    };
}

define_bench_radix_heap!(BenchRadixU8, u8, 9);
define_bench_radix_heap!(BenchRadixU16, u16, 17);
define_bench_radix_heap!(BenchRadixU32, u32, 33);
define_bench_radix_heap!(BenchRadixU64, u64, 65);
define_bench_radix_heap!(BenchRadixU128, u128, 129);

macro_rules! define_bench_signed_radix_heap {
    ($name:ident, $inner:ident, $signed:ty, $unsigned:ty, $sign:expr) => {
        struct $name<T>($inner<T>);

        impl<T> $name<T> {
            fn new() -> Self {
                Self($inner::new())
            }

            #[inline]
            fn len(&self) -> usize {
                self.0.len()
            }

            #[inline]
            fn push(&mut self, key: $signed, value: T) {
                self.0.push((key as $unsigned) ^ $sign, value);
            }

            #[inline]
            fn pop(&mut self) -> Option<($signed, T)> {
                self.0
                    .pop()
                    .map(|(key, value)| (((key ^ $sign) as $signed), value))
            }
        }
    };
}

define_bench_signed_radix_heap!(BenchRadixI8, BenchRadixU8, i8, u8, 1_u8 << 7);
define_bench_signed_radix_heap!(BenchRadixI16, BenchRadixU16, i16, u16, 1_u16 << 15);
define_bench_signed_radix_heap!(BenchRadixI32, BenchRadixU32, i32, u32, 1_u32 << 31);
define_bench_signed_radix_heap!(BenchRadixI64, BenchRadixU64, i64, u64, 1_u64 << 63);
define_bench_signed_radix_heap!(
    BenchRadixI128,
    BenchRadixU128,
    i128,
    u128,
    1_u128 << 127
);

fn bench_radix_signed_self_test() {
    macro_rules! check_signed {
        ($radix:ty, $signed:ty, $unsigned:ty, $sign:expr, $values:expr) => {{
            let values: Vec<$signed> = $values;
            let mut expected = values.clone();
            expected.sort_unstable();
            let mut heap = <$radix>::new();
            for value in values {
                heap.push((value as $unsigned) ^ $sign, ());
            }
            let mut actual = Vec::with_capacity(expected.len());
            while let Some((key, ())) = heap.pop() {
                actual.push((key ^ $sign) as $signed);
            }
            assert_eq!(actual, expected);
        }};
    }
    check_signed!(
        BenchRadixU8<()>,
        i8,
        u8,
        1_u8 << 7,
        vec![i8::MAX, -1, i8::MIN, 0, 1, -100, 100]
    );
    check_signed!(
        BenchRadixU16<()>,
        i16,
        u16,
        1_u16 << 15,
        vec![i16::MAX, -1, i16::MIN, 0, 1, -10_000, 10_000]
    );
    check_signed!(
        BenchRadixU32<()>,
        i32,
        u32,
        1_u32 << 31,
        vec![i32::MAX, -1, i32::MIN, 0, 1, -1_000_000, 1_000_000]
    );
    check_signed!(
        BenchRadixU64<()>,
        i64,
        u64,
        1_u64 << 63,
        vec![i64::MAX, -1, i64::MIN, 0, 1, -1_000_000, 1_000_000]
    );
    check_signed!(
        BenchRadixU128<()>,
        i128,
        u128,
        1_u128 << 127,
        vec![i128::MAX, -1, i128::MIN, 0, 1, -1_000_000, 1_000_000]
    );
}

fn bench_radix_methods() {
    let n = (1 << 18) + 123;
    let q = 180_000;
    let mut rng = BenchRng::new(BENCH_SEED ^ 0x8000);
    let mut random_keys: Vec<u64> = (0..n).map(|_| rng.next() & ((1 << 48) - 1)).collect();
    random_keys.sort_unstable();
    let same_keys = vec![7_u64; n];
    let unique_keys: Vec<u64> = (0..n).map(|index| index as u64 * 17 + 3).collect();

    bench_group(
        "radix_u64_push_only",
        n,
        true,
        vec![
            (
                "binary_heap",
                Box::new(|| {
                    let mut queue = BenchBinaryHeap::with_capacity(n);
                    let start = BenchInstant::now();
                    for &key in &random_keys {
                        queue.push(BenchReverse(key));
                    }
                    bench_black_box(&queue);
                    let elapsed = start.elapsed().as_nanos();
                    (
                        elapsed,
                        bench_mix(queue.len() as u64, queue.peek().unwrap().0),
                    )
                }),
            ),
            (
                "radix_u64",
                Box::new(|| {
                    let mut queue = RadixHeapU64::new();
                    let start = BenchInstant::now();
                    for &key in &random_keys {
                        queue.push(key, ());
                    }
                    bench_black_box(&queue);
                    let elapsed = start.elapsed().as_nanos();
                    (elapsed, bench_mix(queue.len() as u64, random_keys[0]))
                }),
            ),
        ],
    );

    for (distribution, keys) in [
        ("same_bucket0", &same_keys),
        ("unique_small_delta", &unique_keys),
        ("wide", &random_keys),
    ] {
        let label = format!("radix_u64_pop_drain_{distribution}");
        bench_group(
            &label,
            n,
            true,
            vec![
                (
                    "binary_heap",
                    Box::new(|| {
                        let mut queue: BenchBinaryHeap<_> =
                            keys.iter().copied().map(BenchReverse).collect();
                        bench_timed(|| {
                            let mut checksum = 0;
                            while let Some(BenchReverse(key)) = queue.pop() {
                                checksum = bench_mix(checksum, key);
                            }
                            checksum
                        })
                    }),
                ),
                (
                    "radix_u64",
                    Box::new(|| {
                        let mut queue = RadixHeapU64::new();
                        for &key in keys {
                            queue.push(key, ());
                        }
                        bench_timed(|| {
                            let mut checksum = 0;
                            while let Some((key, ())) = queue.pop() {
                                checksum = bench_mix(checksum, key);
                            }
                            checksum
                        })
                    }),
                ),
            ],
        );
    }

    let initial: Vec<u64> = (0..65_536).map(|_| rng.next() % 1_000_000).collect();
    for (distribution, mask) in [("zero", 0_u64), ("small", 255), ("wide", (1 << 32) - 1)] {
        let deltas: Vec<u64> = (0..q).map(|_| rng.next() & mask).collect();
        let label = format!("radix_u64_pop_push_{distribution}");
        bench_group(
            &label,
            q,
            true,
            vec![
                (
                    "binary_heap",
                    Box::new(|| {
                        let mut queue: BenchBinaryHeap<_> =
                            initial.iter().copied().map(BenchReverse).collect();
                        bench_timed(|| {
                            deltas.iter().fold(0, |checksum, &delta| {
                                let BenchReverse(key) = queue.pop().unwrap();
                                queue.push(BenchReverse(key + delta));
                                bench_mix(checksum, key)
                            })
                        })
                    }),
                ),
                (
                    "radix_u64",
                    Box::new(|| {
                        let mut queue = RadixHeapU64::new();
                        for &key in &initial {
                            queue.push(key, ());
                        }
                        bench_timed(|| {
                            deltas.iter().fold(0, |checksum, &delta| {
                                let (key, ()) = queue.pop().unwrap();
                                queue.push(key + delta, ());
                                bench_mix(checksum, key)
                            })
                        })
                    }),
                ),
            ],
        );
    }

    let repetitions = 32;
    bench_group(
        "radix_u64_clear",
        repetitions,
        true,
        vec![
            (
                "binary_heap",
                Box::new(|| {
                    let base: BenchBinaryHeap<_> =
                        random_keys.iter().copied().map(BenchReverse).collect();
                    let mut elapsed = 0;
                    let mut checksum = 0;
                    for _ in 0..repetitions {
                        let mut queue = base.clone();
                        let start = BenchInstant::now();
                        queue.clear();
                        elapsed += start.elapsed().as_nanos();
                        checksum = bench_mix(checksum, queue.len() as u64);
                    }
                    (elapsed, checksum)
                }),
            ),
            (
                "radix_u64",
                Box::new(|| {
                    let mut base = RadixHeapU64::new();
                    for &key in &random_keys {
                        base.push(key, ());
                    }
                    let mut elapsed = 0;
                    let mut checksum = 0;
                    for _ in 0..repetitions {
                        let mut queue = base.clone();
                        let start = BenchInstant::now();
                        queue.clear();
                        elapsed += start.elapsed().as_nanos();
                        checksum = bench_mix(checksum, queue.len() as u64);
                    }
                    (elapsed, checksum)
                }),
            ),
        ],
    );
}

macro_rules! bench_radix_width {
    ($name:literal, $radix:ty, $key:ty, $n:expr, $make:expr) => {{
        let n = $n;
        let mut rng = BenchRng::new(BENCH_SEED ^ <$key>::BITS as u64 ^ 0x9000);
        let mut keys: Vec<$key> = (0..n).map(|_| ($make)(&mut rng)).collect();
        keys.sort_unstable();
        let label = format!("radix_candidate_{}_push", $name);
        bench_group(
            &label,
            n,
            true,
            vec![
                (
                    "binary_heap",
                    Box::new(|| {
                        let mut queue = BenchBinaryHeap::with_capacity(n);
                        let start = BenchInstant::now();
                        for &key in &keys {
                            queue.push(BenchReverse(key));
                        }
                        bench_black_box(&queue);
                        let elapsed = start.elapsed().as_nanos();
                        (elapsed, queue.len() as u64)
                    }),
                ),
                (
                    "radix_candidate",
                    Box::new(|| {
                        let mut queue = <$radix>::new();
                        let start = BenchInstant::now();
                        for &key in &keys {
                            queue.push(key, ());
                        }
                        bench_black_box(&queue);
                        let elapsed = start.elapsed().as_nanos();
                        (elapsed, queue.len() as u64)
                    }),
                ),
            ],
        );
        let label = format!("radix_candidate_{}_pop_drain", $name);
        bench_group(
            &label,
            n,
            true,
            vec![
                (
                    "binary_heap",
                    Box::new(|| {
                        let mut queue: BenchBinaryHeap<_> =
                            keys.iter().copied().map(BenchReverse).collect();
                        bench_timed(|| {
                            let mut checksum = 0;
                            while let Some(BenchReverse(key)) = queue.pop() {
                                checksum = bench_mix(checksum, bench_integer_hash(key as u128));
                            }
                            checksum
                        })
                    }),
                ),
                (
                    "radix_candidate",
                    Box::new(|| {
                        let mut queue = <$radix>::new();
                        for &key in &keys {
                            queue.push(key, ());
                        }
                        bench_timed(|| {
                            let mut checksum = 0;
                            while let Some((key, ())) = queue.pop() {
                                checksum = bench_mix(checksum, bench_integer_hash(key as u128));
                            }
                            checksum
                        })
                    }),
                ),
            ],
        );
    }};
}

fn bench_radix_widths() {
    bench_radix_width!(
        "u8",
        BenchRadixU8<()>,
        u8,
        1 << 18,
        |rng: &mut BenchRng| rng.next() as u8
    );
    bench_radix_width!(
        "u16",
        BenchRadixU16<()>,
        u16,
        1 << 18,
        |rng: &mut BenchRng| rng.next() as u16
    );
    bench_radix_width!(
        "u32",
        BenchRadixU32<()>,
        u32,
        1 << 18,
        |rng: &mut BenchRng| rng.next() as u32
    );
    bench_radix_width!(
        "u64",
        BenchRadixU64<()>,
        u64,
        1 << 18,
        |rng: &mut BenchRng| rng.next()
    );
    bench_radix_width!(
        "u128",
        BenchRadixU128<()>,
        u128,
        1 << 18,
        |rng: &mut BenchRng| { (rng.next() as u128) << 64 | rng.next() as u128 }
    );
    bench_radix_width!(
        "i8",
        BenchRadixI8<()>,
        i8,
        1 << 18,
        |rng: &mut BenchRng| rng.next() as i8
    );
    bench_radix_width!(
        "i16",
        BenchRadixI16<()>,
        i16,
        1 << 18,
        |rng: &mut BenchRng| rng.next() as i16
    );
    bench_radix_width!(
        "i32",
        BenchRadixI32<()>,
        i32,
        1 << 18,
        |rng: &mut BenchRng| rng.next() as i32
    );
    bench_radix_width!(
        "i64",
        BenchRadixI64<()>,
        i64,
        1 << 18,
        |rng: &mut BenchRng| rng.next() as i64
    );
    bench_radix_width!(
        "i128",
        BenchRadixI128<()>,
        i128,
        1 << 18,
        |rng: &mut BenchRng| {
            ((rng.next() as u128) << 64 | rng.next() as u128) as i128
        }
    );
}

macro_rules! bench_radix_monotone_group {
    (
        $label:expr,
        $radix:ty,
        $key:ty,
        $initial:expr,
        $deltas:expr
        $(, bucket = $bucket:ty)?
        $(, dary = $dary:ty)?
        $(,)?
    ) => {{
        let initial_values = &$initial;
        let delta_values = &$deltas;
        let mut verification: BenchBinaryHeap<_> = initial_values
            .iter()
            .copied()
            .map(BenchReverse)
            .collect();
        for &delta in delta_values {
            let BenchReverse(key) = verification.pop().unwrap();
            verification.push(BenchReverse(
                key.checked_add(delta).expect("radix benchmark key overflow"),
            ));
        }

        let mut cases: Vec<BenchCase<'_>> = vec![
            (
                "binary_heap",
                Box::new(|| {
                    let mut queue: BenchBinaryHeap<_> = initial_values
                        .iter()
                        .copied()
                        .map(BenchReverse)
                        .collect();
                    bench_timed(|| {
                        delta_values.iter().fold(0, |checksum, &delta| {
                            let BenchReverse(key) = queue.pop().unwrap();
                            queue.push(BenchReverse(key + delta));
                            bench_mix(checksum, bench_integer_hash(key as u128))
                        })
                    })
                }),
            ),
            (
                "radix_candidate",
                Box::new(|| {
                    let mut queue = <$radix>::new();
                    for &key in initial_values {
                        queue.push(key, ());
                    }
                    bench_timed(|| {
                        delta_values.iter().fold(0, |checksum, &delta| {
                            let (key, ()) = queue.pop().unwrap();
                            queue.push(key + delta, ());
                            bench_mix(checksum, bench_integer_hash(key as u128))
                        })
                    })
                }),
            ),
        ];
        $(
            cases.push((
                "bucket_queue",
                Box::new(|| {
                    let transformed: Vec<$key> = initial_values
                        .iter()
                        .map(|&key| <$key>::MAX - key)
                        .collect();
                    let mut queue = <$bucket>::from(transformed);
                    bench_timed(|| {
                        delta_values.iter().fold(0, |checksum, &delta| {
                            let key = <$key>::MAX - queue.pop().unwrap();
                            queue.push(<$key>::MAX - (key + delta));
                            bench_mix(checksum, bench_integer_hash(key as u128))
                        })
                    })
                }),
            ));
        )?
        $(
            for backend in bench_backends() {
                let name = match backend {
                    SimdBackend::Scalar => "dary_scalar",
                    SimdBackend::Avx2 => "dary_avx2",
                    SimdBackend::Avx512 => "dary_avx512",
                };
                cases.push((
                    name,
                    Box::new(move || {
                        let transformed: Vec<$key> = initial_values
                            .iter()
                            .map(|&key| <$key>::MAX - key)
                            .collect();
                        let mut queue = <$dary>::benchmark_build(transformed, backend);
                        bench_timed(|| {
                            delta_values.iter().fold(0, |checksum, &delta| {
                                let key = <$key>::MAX - queue.pop().unwrap();
                                queue.push(<$key>::MAX - (key + delta));
                                bench_mix(checksum, bench_integer_hash(key as u128))
                            })
                        })
                    }),
                ));
            }
        )?
        let label = $label;
        bench_group(&label, delta_values.len(), true, cases);
    }};
}

fn bench_radix_monotone_small_widths() {
    let mut rng = BenchRng::new(BENCH_SEED ^ 0xc000);
    let initial_u8: Vec<u8> = (0..4_096).map(|_| (rng.next() & 63) as u8).collect();
    for (distribution, mask) in [("zero", 0_u8), ("small", 3), ("wide", 15)] {
        let deltas: Vec<u8> = (0..20_000).map(|_| rng.next() as u8 & mask).collect();
        bench_radix_monotone_group!(
            format!("radix_candidate_u8_pop_push_{distribution}"),
            BenchRadixU8<()>,
            u8,
            initial_u8,
            deltas,
            bucket = BucketQueueU8,
        );
    }

    let initial_u16: Vec<u16> = (0..16_384)
        .map(|_| (rng.next() & 4_095) as u16)
        .collect();
    for (distribution, mask) in [("zero", 0_u16), ("small", 15), ("wide", 1_023)] {
        let deltas: Vec<u16> = (0..60_000).map(|_| rng.next() as u16 & mask).collect();
        bench_radix_monotone_group!(
            format!("radix_candidate_u16_pop_push_{distribution}"),
            BenchRadixU16<()>,
            u16,
            initial_u16,
            deltas,
            bucket = BucketQueueU16,
        );
    }
}

fn bench_radix_monotone_u32() {
    let mut rng = BenchRng::new(BENCH_SEED ^ 0xc032);
    let initial: Vec<u32> = (0..65_536)
        .map(|_| (rng.next() % 1_000_000) as u32)
        .collect();
    for (distribution, mask) in [
        ("zero", 0_u32),
        ("small", 255),
        ("wide", (1 << 24) - 1),
    ] {
        let deltas: Vec<u32> = (0..120_000).map(|_| rng.next() as u32 & mask).collect();
        bench_radix_monotone_group!(
            format!("radix_candidate_u32_pop_push_{distribution}"),
            BenchRadixU32<()>,
            u32,
            initial,
            deltas,
            dary = DaryHeapU32,
        );
    }
}

fn bench_radix_monotone_u64() {
    let mut rng = BenchRng::new(BENCH_SEED ^ 0xc064);
    let initial: Vec<u64> = (0..65_536).map(|_| rng.next() % 1_000_000).collect();
    for (distribution, mask) in [
        ("zero", 0_u64),
        ("small", 255),
        ("wide", (1 << 32) - 1),
    ] {
        let deltas: Vec<u64> = (0..120_000).map(|_| rng.next() & mask).collect();
        bench_radix_monotone_group!(
            format!("radix_candidate_u64_pop_push_{distribution}"),
            BenchRadixU64<()>,
            u64,
            initial,
            deltas,
            dary = DaryHeapU64,
        );
    }
}

fn bench_radix_monotone_u128() {
    let mut rng = BenchRng::new(BENCH_SEED ^ 0xc128);
    let initial: Vec<u128> = (0..65_536)
        .map(|_| ((rng.next() as u128 & ((1_u128 << 32) - 1)) << 64) | rng.next() as u128)
        .collect();
    for distribution in ["zero", "small", "wide"] {
        let deltas: Vec<u128> = (0..80_000)
            .map(|_| match distribution {
                "zero" => 0,
                "small" => (rng.next() & 255) as u128,
                "wide" => {
                    ((rng.next() as u128 & ((1_u128 << 32) - 1)) << 64)
                        | rng.next() as u128
                }
                _ => unreachable!(),
            })
            .collect();
        bench_radix_monotone_group!(
            format!("radix_candidate_u128_pop_push_{distribution}"),
            BenchRadixU128<()>,
            u128,
            initial,
            deltas,
            dary = DaryHeapU128,
        );
    }
}

macro_rules! bench_radix_signed_monotone_group {
    ($label:expr, $radix:ty, $key:ty, $initial:expr, $deltas:expr) => {{
        let initial_values = &$initial;
        let delta_values = &$deltas;
        let mut verification: BenchBinaryHeap<_> = initial_values
            .iter()
            .copied()
            .map(BenchReverse)
            .collect();
        for &delta in delta_values {
            let BenchReverse(key) = verification.pop().unwrap();
            verification.push(BenchReverse(key.checked_add(delta).unwrap()));
        }

        bench_group(
            $label,
            delta_values.len(),
            true,
            vec![
                (
                    "binary_heap",
                    Box::new(|| {
                        let mut queue: BenchBinaryHeap<_> = initial_values
                            .iter()
                            .copied()
                            .map(BenchReverse)
                            .collect();
                        bench_timed(|| {
                            delta_values.iter().fold(0, |checksum, &delta| {
                                let BenchReverse(key) = queue.pop().unwrap();
                                queue.push(BenchReverse(key + delta));
                                bench_mix(checksum, bench_integer_hash(key as u128))
                            })
                        })
                    }),
                ),
                (
                    "radix_candidate_signed",
                    Box::new(|| {
                        let mut queue = <$radix>::new();
                        for &key in initial_values {
                            queue.push(key, ());
                        }
                        bench_timed(|| {
                            delta_values.iter().fold(0, |checksum, &delta| {
                                let (key, ()) = queue.pop().unwrap();
                                queue.push(key + delta, ());
                                bench_mix(checksum, bench_integer_hash(key as u128))
                            })
                        })
                    }),
                ),
            ],
        );
    }};
}

fn bench_radix_signed_monotone() {
    let mut rng = BenchRng::new(BENCH_SEED ^ 0xc1a0);

    let initial_i8: Vec<i8> = (0..4_096)
        .map(|_| ((rng.next() as u8 & 63) ^ (1_u8 << 7)) as i8)
        .collect();
    let deltas_i8: Vec<i8> = (0..20_000).map(|_| (rng.next() & 3) as i8).collect();
    bench_radix_signed_monotone_group!(
        "radix_candidate_i8_pop_push_small",
        BenchRadixI8<()>,
        i8,
        initial_i8,
        deltas_i8
    );

    let initial_i16: Vec<i16> = (0..16_384)
        .map(|_| (((rng.next() & 4_095) as u16) ^ (1_u16 << 15)) as i16)
        .collect();
    let deltas_i16: Vec<i16> = (0..60_000).map(|_| (rng.next() & 15) as i16).collect();
    bench_radix_signed_monotone_group!(
        "radix_candidate_i16_pop_push_small",
        BenchRadixI16<()>,
        i16,
        initial_i16,
        deltas_i16
    );

    let initial_i32: Vec<i32> = (0..65_536)
        .map(|_| (((rng.next() % 1_000_000) as u32) ^ (1_u32 << 31)) as i32)
        .collect();
    let deltas_i32: Vec<i32> = (0..120_000).map(|_| (rng.next() & 255) as i32).collect();
    bench_radix_signed_monotone_group!(
        "radix_candidate_i32_pop_push_small",
        BenchRadixI32<()>,
        i32,
        initial_i32,
        deltas_i32
    );

    let initial_i64: Vec<i64> = (0..65_536)
        .map(|_| ((rng.next() % 1_000_000) ^ (1_u64 << 63)) as i64)
        .collect();
    let deltas_i64: Vec<i64> = (0..120_000).map(|_| (rng.next() & 255) as i64).collect();
    bench_radix_signed_monotone_group!(
        "radix_candidate_i64_pop_push_small",
        BenchRadixI64<()>,
        i64,
        initial_i64,
        deltas_i64
    );

    let initial_i128: Vec<i128> = (0..65_536)
        .map(|_| (((rng.next() % 1_000_000) as u128) ^ (1_u128 << 127)) as i128)
        .collect();
    let deltas_i128: Vec<i128> = (0..80_000).map(|_| (rng.next() & 255) as i128).collect();
    bench_radix_signed_monotone_group!(
        "radix_candidate_i128_pop_push_small",
        BenchRadixI128<()>,
        i128,
        initial_i128,
        deltas_i128
    );
}

fn bench_dijkstra_u32_contract() {
    let n = 30_000;
    let degree = 8;
    let mut rng = BenchRng::new(BENCH_SEED ^ 0xd032);
    let graph: Vec<Vec<(usize, u32)>> = (0..n)
        .map(|vertex| {
            (0..degree)
                .map(|step| {
                    (
                        (vertex + 1 + rng.index(n - 1)) % n,
                        1 + (rng.next() % 1_000) as u32 + step as u32,
                    )
                })
                .collect()
        })
        .collect();

    fn binary(graph: &[Vec<(usize, u32)>]) -> u64 {
        let mut distance = vec![u32::MAX; graph.len()];
        distance[0] = 0;
        let mut queue = BenchBinaryHeap::new();
        queue.push((BenchReverse(0_u32), 0_usize));
        while let Some((BenchReverse(cost), vertex)) = queue.pop() {
            if distance[vertex] != cost {
                continue;
            }
            for &(next, weight) in &graph[vertex] {
                let next_cost = cost + weight;
                if next_cost < distance[next] {
                    distance[next] = next_cost;
                    queue.push((BenchReverse(next_cost), next));
                }
            }
        }
        distance
            .into_iter()
            .fold(0, |checksum, value| bench_mix(checksum, value as u64))
    }

    fn radix(graph: &[Vec<(usize, u32)>]) -> u64 {
        let mut distance = vec![u32::MAX; graph.len()];
        distance[0] = 0;
        let mut queue = BenchRadixU32::new();
        queue.push(0, 0_usize);
        while let Some((cost, vertex)) = queue.pop() {
            if distance[vertex] != cost {
                continue;
            }
            for &(next, weight) in &graph[vertex] {
                let next_cost = cost + weight;
                if next_cost < distance[next] {
                    distance[next] = next_cost;
                    queue.push(next_cost, next);
                }
            }
        }
        distance
            .into_iter()
            .fold(0, |checksum, value| bench_mix(checksum, value as u64))
    }

    fn dary(graph: &[Vec<(usize, u32)>], backend: SimdBackend) -> u64 {
        let mut distance = vec![u32::MAX; graph.len()];
        distance[0] = 0;
        let mut queue = DaryHeapU64::benchmark_empty(0, backend);
        queue.push((u32::MAX as u64) << 32 | u32::MAX as u64);
        while let Some(encoded) = queue.pop() {
            let cost = u32::MAX - (encoded >> 32) as u32;
            let vertex = (u32::MAX - encoded as u32) as usize;
            if distance[vertex] != cost {
                continue;
            }
            for &(next, weight) in &graph[vertex] {
                let next_cost = cost + weight;
                if next_cost < distance[next] {
                    distance[next] = next_cost;
                    queue.push(
                        ((u32::MAX - next_cost) as u64) << 32
                            | (u32::MAX - next as u32) as u64,
                    );
                }
            }
        }
        distance
            .into_iter()
            .fold(0, |checksum, value| bench_mix(checksum, value as u64))
    }

    let mut cases: Vec<BenchCase<'_>> = vec![
        ("binary_heap", Box::new(|| bench_timed(|| binary(&graph)))),
        ("radix_u32_candidate", Box::new(|| bench_timed(|| radix(&graph)))),
    ];
    for backend in bench_backends() {
        let name = match backend {
            SimdBackend::Scalar => "dary_u64_scalar",
            SimdBackend::Avx2 => "dary_u64_avx2",
            SimdBackend::Avx512 => "dary_u64_avx512",
        };
        let graph = &graph;
        cases.push((name, Box::new(move || bench_timed(|| dary(graph, backend)))));
    }
    bench_group("dijkstra_u32_queue_contract", n * degree, true, cases);
}

fn bench_pairing_heap_baseline() {
    type BenchPairingHeap = PairingHeap<u128, comparator::Greater>;

    for n in [4_096, 65_536] {
        let q = 60_000;
        let mut rng = BenchRng::new(BENCH_SEED ^ 0xe000 ^ n as u64);
        let values: Vec<u128> = (0..n)
            .map(|_| (rng.next() as u128) << 64 | rng.next() as u128)
            .collect();
        let updates: Vec<u128> = (0..q)
            .map(|_| (rng.next() as u128) << 64 | rng.next() as u128)
            .collect();

        let label = format!("pairing_baseline_u128_n{n}_build");
        let mut cases: Vec<BenchCase<'_>> = vec![
            (
                "binary_heap",
                Box::new(|| {
                    let input = values.clone();
                    let start = BenchInstant::now();
                    let heap = BenchBinaryHeap::from(input);
                    let elapsed = start.elapsed().as_nanos();
                    (
                        elapsed,
                        bench_mix(
                            heap.len() as u64,
                            bench_integer_hash(*heap.peek().unwrap()),
                        ),
                    )
                }),
            ),
            (
                "pairing_heap",
                Box::new(|| {
                    let input = values.clone();
                    let start = BenchInstant::now();
                    let mut heap: BenchPairingHeap = input.into_iter().collect();
                    let elapsed = start.elapsed().as_nanos();
                    let checksum = bench_mix(
                        heap.len() as u64,
                        bench_integer_hash(*heap.peek().unwrap()),
                    );
                    while heap.pop().is_some() {}
                    (elapsed, checksum)
                }),
            ),
        ];
        for backend in bench_backends() {
            let name = match backend {
                SimdBackend::Scalar => "dary_scalar",
                SimdBackend::Avx2 => "dary_avx2",
                SimdBackend::Avx512 => "dary_avx512",
            };
            let values = &values;
            cases.push((
                name,
                Box::new(move || {
                    let input = values.clone();
                    let start = BenchInstant::now();
                    let heap = DaryHeapU128::benchmark_build(input, backend);
                    let elapsed = start.elapsed().as_nanos();
                    (
                        elapsed,
                        bench_mix(
                            heap.len() as u64,
                            bench_integer_hash(heap.peek().unwrap()),
                        ),
                    )
                }),
            ));
        }
        bench_group(&label, n, true, cases);

        let label = format!("pairing_baseline_u128_n{n}_pop_drain");
        let mut cases: Vec<BenchCase<'_>> = vec![
            (
                "binary_heap",
                Box::new(|| {
                    let mut heap = BenchBinaryHeap::from(values.clone());
                    bench_timed(|| {
                        let mut checksum = 0;
                        while let Some(value) = heap.pop() {
                            checksum = bench_mix(checksum, bench_integer_hash(value));
                        }
                        checksum
                    })
                }),
            ),
            (
                "pairing_heap",
                Box::new(|| {
                    let mut heap: BenchPairingHeap = values.iter().copied().collect();
                    bench_timed(|| {
                        let mut checksum = 0;
                        while let Some(value) = heap.pop() {
                            checksum = bench_mix(checksum, bench_integer_hash(value));
                        }
                        checksum
                    })
                }),
            ),
        ];
        for backend in bench_backends() {
            let name = match backend {
                SimdBackend::Scalar => "dary_scalar",
                SimdBackend::Avx2 => "dary_avx2",
                SimdBackend::Avx512 => "dary_avx512",
            };
            let values = &values;
            cases.push((
                name,
                Box::new(move || {
                    let mut heap = DaryHeapU128::benchmark_build(values.clone(), backend);
                    bench_timed(|| {
                        let mut checksum = 0;
                        while let Some(value) = heap.pop() {
                            checksum = bench_mix(checksum, bench_integer_hash(value));
                        }
                        checksum
                    })
                }),
            ));
        }
        bench_group(&label, n, true, cases);

        for operation in ["pop_push", "replace", "top_k"] {
            let label = format!("pairing_baseline_u128_n{n}_{operation}");
            let mut cases: Vec<BenchCase<'_>> = vec![
                (
                    "binary_heap",
                    Box::new(|| {
                        let mut heap = BenchBinaryHeap::from(values.clone());
                        bench_timed(|| {
                            updates.iter().fold(0, |checksum, &value| {
                                if operation == "top_k" && value >= *heap.peek().unwrap() {
                                    return bench_mix(checksum, 0);
                                }
                                let removed = heap.pop().unwrap();
                                heap.push(value);
                                bench_mix(checksum, bench_integer_hash(removed))
                            })
                        })
                    }),
                ),
                (
                    "pairing_heap",
                    Box::new(|| {
                        let mut heap: BenchPairingHeap = values.iter().copied().collect();
                        let result = bench_timed(|| {
                            updates.iter().fold(0, |checksum, &value| {
                                if operation == "top_k" && value >= *heap.peek().unwrap() {
                                    return bench_mix(checksum, 0);
                                }
                                let removed = *heap.peek().unwrap();
                                if operation == "pop_push" {
                                    heap.pop();
                                    heap.push(value);
                                } else {
                                    *heap.peek_mut().unwrap() = value;
                                }
                                bench_mix(checksum, bench_integer_hash(removed))
                            })
                        });
                        while heap.pop().is_some() {}
                        result
                    }),
                ),
            ];
            for backend in bench_backends() {
                let name = match backend {
                    SimdBackend::Scalar => "dary_scalar",
                    SimdBackend::Avx2 => "dary_avx2",
                    SimdBackend::Avx512 => "dary_avx512",
                };
                let values = &values;
                let updates = &updates;
                cases.push((
                    name,
                    Box::new(move || {
                        let mut heap = DaryHeapU128::benchmark_build(values.clone(), backend);
                        bench_timed(|| {
                            updates.iter().fold(0, |checksum, &value| {
                                if operation == "top_k" && value >= heap.peek().unwrap() {
                                    return bench_mix(checksum, 0);
                                }
                                let removed = if operation == "pop_push" {
                                    let removed = heap.pop().unwrap();
                                    heap.push(value);
                                    removed
                                } else {
                                    heap.replace(value).unwrap()
                                };
                                bench_mix(checksum, bench_integer_hash(removed))
                            })
                        })
                    }),
                ));
            }
            bench_group(&label, q, true, cases);
        }
    }
}

fn bench_queue_contracts() {
    let n = 65_536;
    let q = 180_000;
    let mut rng = BenchRng::new(BENCH_SEED ^ 0xa000);
    let initial: Vec<u64> = (0..n).map(|_| rng.next() % 1_000_000).collect();
    for (distribution, mask) in [("small_delta", 255_u64), ("wide_delta", (1 << 32) - 1)] {
        let deltas: Vec<_> = (0..q).map(|_| rng.next() & mask).collect();
        let label = format!("min_queue_contract_{distribution}");
        let mut cases: Vec<BenchCase<'_>> = vec![
            (
                "binary_heap",
                Box::new(|| {
                    let mut queue: BenchBinaryHeap<_> =
                        initial.iter().copied().map(BenchReverse).collect();
                    bench_timed(|| {
                        deltas.iter().fold(0, |checksum, &delta| {
                            let BenchReverse(key) = queue.pop().unwrap();
                            queue.push(BenchReverse(key + delta));
                            bench_mix(checksum, key)
                        })
                    })
                }),
            ),
            (
                "radix_u64",
                Box::new(|| {
                    let mut queue = RadixHeapU64::new();
                    for &key in &initial {
                        queue.push(key, ());
                    }
                    bench_timed(|| {
                        deltas.iter().fold(0, |checksum, &delta| {
                            let (key, ()) = queue.pop().unwrap();
                            queue.push(key + delta, ());
                            bench_mix(checksum, key)
                        })
                    })
                }),
            ),
        ];
        for backend in bench_backends() {
            let name = match backend {
                SimdBackend::Scalar => "dary_scalar",
                SimdBackend::Avx2 => "dary_avx2",
                SimdBackend::Avx512 => "dary_avx512",
            };
            let initial = &initial;
            let deltas = &deltas;
            cases.push((
                name,
                Box::new(move || {
                    let transformed: Vec<_> = initial.iter().map(|&key| u64::MAX - key).collect();
                    let mut queue = DaryHeapU64::benchmark_build(transformed, backend);
                    bench_timed(|| {
                        deltas.iter().fold(0, |checksum, &delta| {
                            let key = u64::MAX - queue.pop().unwrap();
                            queue.push(u64::MAX - (key + delta));
                            bench_mix(checksum, key)
                        })
                    })
                }),
            ));
        }
        bench_group(&label, q, true, cases);
    }
    bench_dijkstra_contracts();
}

fn bench_dijkstra_contracts() {
    let n = 30_000;
    let degree = 8;
    let mut rng = BenchRng::new(BENCH_SEED ^ 0xb000);
    let graph: Vec<Vec<(usize, u64)>> = (0..n)
        .map(|vertex| {
            (0..degree)
                .map(|step| {
                    (
                        (vertex + 1 + rng.index(n - 1)) % n,
                        1 + (rng.next() % 1_000) + step as u64,
                    )
                })
                .collect()
        })
        .collect();

    fn binary(graph: &[Vec<(usize, u64)>]) -> u64 {
        let mut distance = vec![u64::MAX; graph.len()];
        distance[0] = 0;
        let mut queue = BenchBinaryHeap::new();
        queue.push((BenchReverse(0_u64), 0_usize));
        while let Some((BenchReverse(cost), vertex)) = queue.pop() {
            if distance[vertex] != cost {
                continue;
            }
            for &(next, weight) in &graph[vertex] {
                let next_cost = cost + weight;
                if next_cost < distance[next] {
                    distance[next] = next_cost;
                    queue.push((BenchReverse(next_cost), next));
                }
            }
        }
        distance.into_iter().fold(0, bench_mix)
    }

    fn radix(graph: &[Vec<(usize, u64)>]) -> u64 {
        let mut distance = vec![u64::MAX; graph.len()];
        distance[0] = 0;
        let mut queue = RadixHeapU64::new();
        queue.push(0, 0_usize);
        while let Some((cost, vertex)) = queue.pop() {
            if distance[vertex] != cost {
                continue;
            }
            for &(next, weight) in &graph[vertex] {
                let next_cost = cost + weight;
                if next_cost < distance[next] {
                    distance[next] = next_cost;
                    queue.push(next_cost, next);
                }
            }
        }
        distance.into_iter().fold(0, bench_mix)
    }

    fn dary(graph: &[Vec<(usize, u64)>], backend: SimdBackend) -> u64 {
        let mut distance = vec![u64::MAX; graph.len()];
        distance[0] = 0;
        let mut queue = DaryHeapU128::benchmark_empty(0, backend);
        queue.push((u64::MAX as u128) << 64 | u64::MAX as u128);
        while let Some(encoded) = queue.pop() {
            let cost = u64::MAX - (encoded >> 64) as u64;
            let vertex = (u64::MAX - encoded as u64) as usize;
            if distance[vertex] != cost {
                continue;
            }
            for &(next, weight) in &graph[vertex] {
                let next_cost = cost + weight;
                if next_cost < distance[next] {
                    distance[next] = next_cost;
                    queue.push(
                        ((u64::MAX - next_cost) as u128) << 64 | (u64::MAX - next as u64) as u128,
                    );
                }
            }
        }
        distance.into_iter().fold(0, bench_mix)
    }

    let mut cases: Vec<BenchCase<'_>> = vec![
        ("binary_heap", Box::new(|| bench_timed(|| binary(&graph)))),
        ("radix_u64", Box::new(|| bench_timed(|| radix(&graph)))),
    ];
    for backend in bench_backends() {
        let name = match backend {
            SimdBackend::Scalar => "dary_u128_scalar",
            SimdBackend::Avx2 => "dary_u128_avx2",
            SimdBackend::Avx512 => "dary_u128_avx512",
        };
        let graph = &graph;
        cases.push((name, Box::new(move || bench_timed(|| dary(graph, backend)))));
    }
    bench_group("dijkstra_queue_contract", n * degree, true, cases);
}

fn main() {
    bench_configure_runner();
    let mut input = String::new();
    bench_io::stdin().read_to_string(&mut input).unwrap();
    let suite = input
        .split_whitespace()
        .next()
        .unwrap_or("0")
        .parse()
        .unwrap();
    bench_header(suite);
    match suite {
        0 => {
            bench_self_test();
            bench_diagnostics();
        }
        10 => bench_bit_vector_queries(),
        11 => bench_bit_vector_build_select(),
        12 => bench_wavelet_queries(),
        13 => bench_wavelet_batch(),
        14 => bench_wavelet_fold(),
        15 => bench_wavelet_existing_baseline(),
        20 => bench_static_small(),
        21 => bench_static_medium(),
        22 => bench_static_large(),
        30 => bench_bucket_queue(),
        31 => bench_dary_u32_small(),
        32 => bench_dary_u64_small(),
        33 => bench_dary_u128_small(),
        34 => bench_dary_u32_large(),
        35 => bench_dary_u32_variants(),
        36 => bench_dary_u64_large(),
        37 => bench_dary_u64_variants(),
        38 => bench_dary_u128_large(),
        39 => bench_dary_u128_variants(),
        40 => bench_wide_prefix_u32(),
        41 => bench_wide_prefix_u64(),
        50 => bench_wide_segment_i32(),
        51 => bench_wide_segment_i64(),
        60 => bench_radix_methods(),
        61 => bench_radix_widths(),
        62 => bench_queue_contracts(),
        63 => bench_radix_monotone_small_widths(),
        64 => bench_radix_monotone_u32(),
        65 => bench_radix_monotone_u64(),
        66 => bench_radix_monotone_u128(),
        67 => bench_dijkstra_u32_contract(),
        68 => bench_pairing_heap_baseline(),
        69 => bench_radix_signed_monotone(),
        _ => panic!("unknown suite"),
    }
}
