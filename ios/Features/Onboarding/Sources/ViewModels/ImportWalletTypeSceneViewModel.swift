import Components
import Foundation
import class Gemstone.GemChainService
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct ImportWalletTypeSceneViewModel {
    private let allChains: [Chain]

    public init() {
        allChains = GemChainService.shared.getChains(query: .empty).map { Chain(core: $0) }
    }

    var multicoinListItem: ListItemModel {
        ListItemModel(title: Localized.Wallet.multicoin, imageStyle: .asset(assetImage: AssetImage.image(Images.Logo.logo)))
    }

    func listItem(for chain: Chain) -> ListItemModel {
        ListItemModel(title: chain.networkName, imageStyle: .asset(assetImage: AssetImage.image(ChainImage(chain: chain).image)))
    }

    var title: String {
        Localized.Wallet.Import.title
    }

    func items(for searchText: String) -> [Chain] {
        searchText.isEmpty ? allChains : GemChainService.shared.getChains(query: searchText).map { Chain(core: $0) }
    }
}

// MARK: - Equatable

extension ImportWalletTypeSceneViewModel: Equatable {}

// MARK: - Hashable

extension ImportWalletTypeSceneViewModel: Hashable {
    public func hash(into hasher: inout Hasher) {
        hasher.combine(allChains)
    }
}
