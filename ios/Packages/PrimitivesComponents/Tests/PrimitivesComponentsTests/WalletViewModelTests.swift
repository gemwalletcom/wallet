import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing

struct WalletViewModelTests {
    @Test
    func imageUsesChainImageForSingleChainWallets() {
        let wallet = Wallet.mock(
            id: .single(chain: .seiEvm, address: "0x1"),
            type: .single,
            accounts: [.mock(chain: .seiEvm)],
        )

        #expect(WalletViewModel(wallet: wallet).image == ChainImage(chain: .seiEvm).image)
    }
}
