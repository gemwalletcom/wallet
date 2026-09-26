// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemAssetIcon
import Primitives
import PrimitivesComponents
import Testing

struct AssetImageIconTests {
    @Test
    func aLocalCoinDrawsItsChainWithTheBadgeCoreChose() {
        let icon = GemAssetIcon(image: .local(chain: Chain.ethereum.rawValue), badge: Chain.arbitrum.rawValue, placeholder: "ERC20")

        #expect(AssetImage(icon: icon) == AssetImage(
            type: .text("ERC20"),
            placeholder: ChainImage(chain: .ethereum).image,
            chainPlaceholder: ChainImage(chain: .arbitrum).image,
        ))
    }

    @Test
    func aKnownStablecoinDrawsTheBundledLogo() {
        let icon = GemAssetIcon(image: .localToken(token: .usdt), badge: Chain.ethereum.rawValue, placeholder: "ERC20")

        #expect(AssetImage(icon: icon) == AssetImage(
            type: .text("ERC20"),
            placeholder: TokenImage(token: .usdt).image,
            chainPlaceholder: ChainImage(chain: .ethereum).image,
        ))
    }

    @Test
    func aRemoteTokenLoadsItsLogoWithNoPlaceholderImage() {
        let url = "https://assets.gemwallet.com/blockchains/base/assets/0x833589fcd6edb6e08f4c7c32d4f71b54bda02913/logo.png"
        let icon = GemAssetIcon(image: .remote(url: url), badge: Chain.base.rawValue, placeholder: nil)

        #expect(AssetImage(icon: icon) == AssetImage(
            type: .text(.empty),
            imageURL: URL(string: url),
            placeholder: .none,
            chainPlaceholder: ChainImage(chain: .base).image,
        ))
    }
}
