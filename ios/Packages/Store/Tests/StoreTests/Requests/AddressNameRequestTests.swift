// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct AddressNameRequestTests {
    @Test
    func anOwnWalletNameShowsTheWalletImage() throws {
        let db = DB.mockWithChains([.ethereum])
        let addressStore = AddressStore.mock(db: db)
        let walletAddress = "0x2Df1c51E09aECF9cacB7bc98cB1742757f163dF7"
        let contactAddress = "0x1111111111111111111111111111111111111111"
        try WalletStore(db: db).addWallet(.mock(name: "Savings", accounts: [.mock(chain: .ethereum, address: walletAddress)], imageUrl: "savings.png"))
        try addressStore.updateAddressNames([
            .mock(.mock(chain: .ethereum, address: walletAddress, name: "Savings", type: .internalWallet)),
            .mock(.mock(chain: .ethereum, address: contactAddress, name: "Alice", type: .contact, imageUrl: "alice.png")),
        ])

        #expect(try addressStore.getAddressName(chain: .ethereum, address: walletAddress)?.imageUrl == "savings.png")
        #expect(try addressStore.getAddressName(chain: .ethereum, address: contactAddress)?.imageUrl == "alice.png")
    }
}
