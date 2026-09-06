import Foundation
import protocol Gemstone.GemChainServiceProtocol
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import SwiftUI
import GemstoneServices

public struct ImportWalletTypeViewModel {
    private let preferences: ObservablePreferences
    private let service: any GemChainServiceProtocol

    public init(preferences: ObservablePreferences, service: any GemChainServiceProtocol) {
        self.preferences = preferences
        self.service = service
    }

    func filterChains(for query: String) -> [Chain] {
        service.getChains(query: query).map { Chain(core: $0) }
    }

    var title: String {
        Localized.Wallet.Import.title
    }

    func items(for searchText: String) -> [Chain] {
        filterChains(for: searchText)
    }

    func acceptTerms() {
        preferences.acceptTerms()
    }
}

// MARK: - Equatable

extension ImportWalletTypeViewModel: Equatable {
    public static func == (lhs: ImportWalletTypeViewModel, rhs: ImportWalletTypeViewModel) -> Bool {
        lhs.filterChains(for: "") == rhs.filterChains(for: "")
    }
}

// MARK: - Hashable

extension ImportWalletTypeViewModel: Hashable {
    public func hash(into hasher: inout Hasher) {
        hasher.combine(filterChains(for: ""))
    }
}
