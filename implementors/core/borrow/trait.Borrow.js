(function() {var implementors = {};
implementors["bstr"] = [{"text":"impl Borrow&lt;BStr&gt; for BString","synthetic":false,"types":[]}];
implementors["smallvec"] = [{"text":"impl&lt;A:&nbsp;Array&gt; Borrow&lt;[&lt;A as Array&gt;::Item]&gt; for SmallVec&lt;A&gt;","synthetic":false,"types":[]}];
implementors["spinoso_array"] = [{"text":"impl&lt;T&gt; Borrow&lt;[T]&gt; for SmallArray&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl&lt;T&gt; Borrow&lt;[T]&gt; for Array&lt;T&gt;","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()