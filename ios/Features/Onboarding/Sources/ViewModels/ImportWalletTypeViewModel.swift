import Foundation
import protocol Gemstone.GemChainServiceProtocol
import Localization
import Primitives
import PrimitivesComponents
import SwiftUI
import GemstoneServices

public struct ImportWalletTypeViewModel {
    let walletService: WalletService
    public let chainService: any GemChainServiceProtocol

    public init(walletService: WalletService, chainService: any GemChainServiceProtocol) {
        self.walletService = walletService
        self.chainService = chainService
    }

    var title: String {
        Localized.Wallet.Import.title
    }

    func items(for searchText: String) -> [Chain] {
        filterChains(for: searchText)
    }

    func acceptTerms() {
        walletService.acceptTerms()
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

// MARK: - ChainFilterable

extension ImportWalletTypeViewModel: ChainFilterable {}
