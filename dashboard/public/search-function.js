                const performSearch = async () => {
                    if (!searchQuery.value.trim()) return;
                    
                    loading.value = true;
                    searchPerformed.value = true;
                    const startTime = Date.now();
                    
                    try {
                        const requestBody = {
                            collection: selectedCollection.value || "gov-bips",
                            query: searchQuery.value,
                            limit: 10
                        };
                        
                        const response = await fetch('http://localhost:15003', {
                            method: 'POST',
                            headers: {
                                'Content-Type': 'application/json',
                            },
                            body: JSON.stringify({
                                method: 'vectorizer.VectorizerService/Search',
                                data: requestBody
                            })
                        });
                        
                        if (!response.ok) {
                            throw new Error(`HTTP error! status: ${response.status}`);
                        }
                        
                        const data = await response.json();
                        searchResults.value = data.results || [];
                        searchTime.value = Date.now() - startTime;
                        
                    } catch (error) {
                        console.error('Search error:', error);
                        searchResults.value = [];
                        searchTime.value = Date.now() - startTime;
                        
                        // Fallback: try direct GRPC call simulation
                        try {
                            const grpcResponse = await fetch('http://localhost:15003', {
                                method: 'POST',
                                headers: {
                                    'Content-Type': 'application/json',
                                },
                                body: JSON.stringify({
                                    collection: selectedCollection.value || "gov-bips",
                                    query: searchQuery.value,
                                    limit: 10
                                })
                            });
                            
                            if (grpcResponse.ok) {
                                const grpcData = await grpcResponse.json();
                                searchResults.value = grpcData.results || [];
                            }
                        } catch (grpcError) {
                            console.error('GRPC fallback error:', grpcError);
                        }
                    } finally {
                        loading.value = false;
                    }
                };
