import Testing
@testable import WalletConnectorService
import WalletConnectSign

struct SessionNamespaceWalletConnectorServiceTests {
    @Test
    func addingAccountExtendsItsNamespace() throws {
        let ethereum = try #require(Account("eip155:1:0xabc"))
        let blast = try #require(Account("eip155:81457:0xabc"))
        let solana = try #require(Account("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp:address"))
        let namespaces = [
            "eip155": SessionNamespace(chains: [ethereum.blockchain], accounts: [ethereum], methods: ["eth_sendTransaction"], events: ["chainChanged"]),
            "solana": SessionNamespace(accounts: [solana], methods: ["solana_signMessage"], events: []),
        ]

        let updated = namespaces.adding(blast)

        #expect(updated["eip155"]?.chains == [ethereum.blockchain, blast.blockchain])
        #expect(updated["eip155"]?.accounts == [ethereum, blast])
        #expect(updated["eip155"]?.methods == ["eth_sendTransaction"])
        #expect(updated["eip155"]?.events == ["chainChanged"])
        #expect(updated["solana"] == namespaces["solana"])
    }
}
