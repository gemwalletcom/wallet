import Foundation
import class Gemstone.GemChainService
import GemstonePrimitives
import Localization
import Primitives
import SwiftUI

public struct ImportWalletTypeViewModel {
    private let allChains: [Chain]

    public init() {
        allChains = GemChainService.shared.getChains(query: .empty).map { Chain(core: $0) }
    }

    var title: String {
        Localized.Wallet.Import.title
    }

    func items(for searchText: String) -> [Chain] {
        searchText.isEmpty ? allChains : GemChainService.shared.getChains(query: searchText).map { Chain(core: $0) }
    }
}

// MARK: - Equatable

extension ImportWalletTypeViewModel: Equatable {
    public static func == (lhs: ImportWalletTypeViewModel, rhs: ImportWalletTypeViewModel) -> Bool {
        lhs.allChains == rhs.allChains
    }
}

// MARK: - Hashable

extension ImportWalletTypeViewModel: Hashable {
    public func hash(into hasher: inout Hasher) {
        hasher.combine(allChains)
    }
}
