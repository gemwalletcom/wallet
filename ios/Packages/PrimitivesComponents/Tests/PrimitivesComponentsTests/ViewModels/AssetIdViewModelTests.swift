// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Testing

struct AssetIdViewModelTests {
    @Test
    func networkAssetImage() {
        #expect(
            AssetIdViewModel(assetId: .mock(.bitcoin)).networkAssetImage == AssetImage(
                type: .text(.empty),
                placeholder: ChainImage(chain: .bitcoin).image,
            ),
        )
        #expect(
            AssetIdViewModel(assetId: .mock(.ethereum)).networkAssetImage == AssetImage(
                type: .text(.empty),
                placeholder: ChainImage(chain: .ethereum).image,
            ),
        )
        #expect(
            AssetIdViewModel(assetId: .mock(.arbitrum)).networkAssetImage == AssetImage(
                type: .text(.empty),
                placeholder: ChainImage(chain: .arbitrum).image,
            ),
        )
    }

    @Test
    func knownStablecoinsDrawTheBundledLogo() {
        let usdt = AssetId(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7")
        let usdc = AssetId(chain: .solana, tokenId: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")

        #expect(
            AssetIdViewModel(assetId: usdt).assetImage == AssetImage(
                type: .text("ERC20"),
                placeholder: TokenImage(token: .usdt).image,
                chainPlaceholder: ChainImage(chain: .ethereum).image,
            ),
        )
        #expect(
            AssetIdViewModel(assetId: usdc).assetImage == AssetImage(
                type: .text("SPL"),
                placeholder: TokenImage(token: .usdc).image,
                chainPlaceholder: ChainImage(chain: .solana).image,
            ),
        )
    }

    @Test
    func assetImage() {
        #expect(
            AssetIdViewModel(assetId: .mock(.bitcoin)).assetImage == AssetImage(
                type: .text(.empty),
                placeholder: ChainImage(chain: .bitcoin).image,
            ),
        )
        #expect(
            AssetIdViewModel(assetId: .mock(.ethereum)).assetImage == AssetImage(
                type: .text("ERC20"),
                placeholder: ChainImage(chain: .ethereum).image,
            ),
        )
        #expect(
            AssetIdViewModel(assetId: .mock(.arbitrum)).assetImage == AssetImage(
                type: .text("ERC20"),
                placeholder: ChainImage(chain: .ethereum).image,
                chainPlaceholder: ChainImage(chain: .arbitrum).image,
            ),
        )
        #expect(
            AssetIdViewModel(assetId: .mock(.robinhood)).assetImage == AssetImage(
                type: .text("ERC20"),
                placeholder: ChainImage(chain: .ethereum).image,
                chainPlaceholder: ChainImage(chain: .robinhood).image,
            ),
        )
        let baseUSDC = AssetId(chain: .base, tokenId: "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913")
        #expect(
            AssetIdViewModel(assetId: baseUSDC).assetImage == AssetImage(
                type: .text("ERC20"),
                imageURL: URL(string: "https://assets.gemwallet.com/blockchains/base/assets/0x833589fcd6edb6e08f4c7c32d4f71b54bda02913/logo.png"),
                placeholder: .none,
                chainPlaceholder: ChainImage(chain: .base).image,
            ),
        )
    }

    @Test
    func assetImageHyperCorePerpetual() {
        let btcPerpetualAssetId = AssetId(chain: .hyperCore, tokenId: "perpetual::BTC")
        let btcPerpetualImage = AssetIdViewModel(assetId: btcPerpetualAssetId).assetImage

        #expect(btcPerpetualImage.type == .text("TOKEN"))
        #expect(btcPerpetualImage.placeholder == ChainImage(chain: .bitcoin).image)
        #expect(btcPerpetualImage.chainPlaceholder == ChainImage(chain: .hyperCore).image)

        let ethPerpetualAssetId = AssetId(chain: .hyperCore, tokenId: "perpetual::ETH")
        let ethPerpetualImage = AssetIdViewModel(assetId: ethPerpetualAssetId).assetImage

        #expect(ethPerpetualImage.type == .text("TOKEN"))
        #expect(ethPerpetualImage.placeholder == ChainImage(chain: .ethereum).image)
        #expect(ethPerpetualImage.chainPlaceholder == ChainImage(chain: .hyperCore).image)
    }
}
