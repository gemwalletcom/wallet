// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Testing

struct Wallet_PrimitivesTests {
    @Test
    func canSign() {
        #expect(Wallet.mock(type: .multicoin).canSign == true)
        #expect(Wallet.mock(type: .view).canSign == false)
    }

    @Test
    func walletIdFromType() throws {
        #expect(throws: Error.self) {
            try WalletId.from(type: .multicoin, accounts: [.mock(chain: .bitcoin, address: "0x123")])
        }
        #expect(try WalletId.from(type: .multicoin, accounts: [.mock(chain: .ethereum, address: "0x123")]) == .multicoin(address: "0x123"))
        #expect(try WalletId.from(type: .single, accounts: [.mock(chain: .ethereum, address: "0x456")]) == .single(chain: .ethereum, address: "0x456"))
        #expect(try WalletId.from(type: .privateKey, accounts: [.mock(chain: .bitcoin, address: "bc1abc")]) == .privateKey(chain: .bitcoin, address: "bc1abc"))
        #expect(try WalletId.from(type: .view, accounts: [.mock(chain: .ethereum, address: "0x789")]) == .view(chain: .ethereum, address: "0x789"))
    }
}
