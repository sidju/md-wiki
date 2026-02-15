// Simple search functionality for md-wiki
(function() {
    let searchIndex = null;
    
    // Load the index from embedded data
    async function loadIndex() {
        if (searchIndex) return searchIndex;
        
        // Use the embedded search index data
        if (window.SEARCH_INDEX_DATA) {
            searchIndex = window.SEARCH_INDEX_DATA;
            return searchIndex;
        }
        
        console.error('Search index data not found. Make sure search-data.js is included before search.js');
        return null;
    }
    
    // Search the index for matching documents and headings
    function search(query) {
        if (!searchIndex || !query) return [];
        
        const lowerQuery = query.toLowerCase();
        const results = [];
        
        // Search through all documents
        for (const doc of searchIndex.documents) {
            const docTitle = doc.path.replace('.html', '').replace(/-/g, ' ');
            const docMatches = docTitle.toLowerCase().includes(lowerQuery);
            
            // Add document match
            if (docMatches) {
                results.push({
                    type: 'document',
                    title: docTitle,
                    path: doc.path,
                    heading: null
                });
            }
            
            // Search through headings in this document
            for (const heading of doc.headings) {
                if (heading.text.toLowerCase().includes(lowerQuery)) {
                    results.push({
                        type: 'heading',
                        title: docTitle,
                        path: doc.path,
                        heading: heading
                    });
                }
            }
        }
        
        return results;
    }
    
    // Render search results
    function renderResults(results) {
        const resultsContainer = document.getElementById('search-results');
        
        if (!results || results.length === 0) {
            resultsContainer.classList.remove('active');
            resultsContainer.innerHTML = '';
            return;
        }
        
        resultsContainer.innerHTML = '';
        resultsContainer.classList.add('active');
        
        results.slice(0, 10).forEach(result => {
            const item = document.createElement('div');
            item.className = 'search-result-item';
            
            const titleSpan = document.createElement('span');
            titleSpan.className = 'search-result-title';
            titleSpan.textContent = result.title;
            item.appendChild(titleSpan);
            
            if (result.heading) {
                const headingSpan = document.createElement('span');
                headingSpan.className = 'search-result-heading';
                headingSpan.textContent = ' → ' + result.heading.text;
                item.appendChild(headingSpan);
            }
            
            item.addEventListener('click', () => {
                if (result.heading) {
                    window.location.href = result.path + '#' + result.heading.id;
                } else {
                    window.location.href = result.path;
                }
            });
            
            resultsContainer.appendChild(item);
        });
    }
    
    // Initialize search when DOM is ready
    function initSearch() {
        const searchInput = document.getElementById('search-input');
        const resultsContainer = document.getElementById('search-results');
        
        if (!searchInput || !resultsContainer) return;
        
        // Load index
        loadIndex();
        
        // Handle input
        let searchTimeout;
        searchInput.addEventListener('input', (e) => {
            clearTimeout(searchTimeout);
            const query = e.target.value.trim();
            
            if (!query) {
                renderResults([]);
                return;
            }
            
            searchTimeout = setTimeout(() => {
                const results = search(query);
                renderResults(results);
            }, 300);
        });
        
        // Close results when clicking outside
        document.addEventListener('click', (e) => {
            if (!searchInput.contains(e.target) && !resultsContainer.contains(e.target)) {
                resultsContainer.classList.remove('active');
            }
        });
        
        // Handle keyboard navigation
        searchInput.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                resultsContainer.classList.remove('active');
                searchInput.blur();
            }
        });
    }
    
    // Initialize when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initSearch);
    } else {
        initSearch();
    }
})();
