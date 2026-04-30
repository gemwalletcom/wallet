import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing

struct WalletViewModelTests {
    @Test
    func imageUsesChainImageForSingleChainWallets() {
        let wallet = Wallet.mock(
            type: .single,
            accounts: [.mock(chain: .seiEvm)],
        )

        #expect(WalletViewModel(wallet: wallet).image == ChainImage(chain: .seiEvm).placeholder)
    }
}
