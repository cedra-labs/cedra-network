Oracles flow
1.Fetch prices from http endpoint
// 2 & 3 must be executed async in parallel(starts at the same time and waits other
to complete(tps of price changes depends on how fast this 2 tasks fully completed))
2.Save prices in memory-cache  
3.Generate txn to set/update storage in blockchain & call move function to save prices
4.Check if memory-cache timestamp same as blockchain one
5.Use price for gas computation 
