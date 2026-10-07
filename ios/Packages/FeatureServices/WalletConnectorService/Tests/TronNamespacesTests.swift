import class Gemstone.GemChainService
import Primitives
import PrimitivesTestKit
import Testing
@testable import WalletConnectorService
import WalletConnectorServiceTestKit
import WalletConnectSign

struct TronNamespacesTests {
    private let chainService = GemChainService()
    private let account = Primitives.Account.mock(chain: .tron, address: "address")

    @Test(arguments: ["tron:728126428", "tron:0x2b6653dc"], [false, true])
    func approvesRequestedReference(chainId: String, optional: Bool) throws {
        let namespaces = try AutoNamespaces.build(
            sessionProposal: .mock(chainId: chainId, optional: optional),
            chains: account.chain.blockchains(chainService: chainService),
            methods: ["tron_signMessage"],
            events: [],
            accounts: account.blockchains(chainService: chainService),
        )
        #expect(namespaces["tron"]?.chains?.map(\.absoluteString) == [chainId])
        #expect(namespaces["tron"]?.accounts.map(\.absoluteString) == ["\(chainId):\(account.address)"])
    }
}
