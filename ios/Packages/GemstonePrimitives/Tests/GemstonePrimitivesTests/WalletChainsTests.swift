// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing

final class WalletChainsTests {
    @Test
    func chainsAll() {
        let wallet = Wallet.mock(accounts: [
            .mock(chain: .bitcoin),
            .mock(chain: .doge),
            .mock(chain: .ethereum),
        ])

        let result = wallet.chains
        let expectedChains: [Chain] = [.bitcoin, .ethereum, .doge]
        #expect(result == expectedChains)
    }

    @Test
    func hasTokenSupport() {
        #expect(Wallet.mock(accounts: [.mock(chain: .bitcoin), .mock(chain: .doge)]).hasTokenSupport == false)
        #expect(Wallet.mock(accounts: [.mock(chain: .bitcoin), .mock(chain: .stellar)]).hasTokenSupport == true)
    }

    @Test
    func chainSorting() {
        let wallet = Wallet.mock(accounts: [
            .mock(chain: .doge),
            .mock(chain: .ethereum),
            .mock(chain: .bitcoin),
        ])

        let result = wallet.chains
        let expectedChains: [Chain] = [.bitcoin, .ethereum, .doge]
        #expect(result == expectedChains)
    }
}
