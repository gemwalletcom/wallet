import Components
import Foundation
import class Gemstone.GemAddressService
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

public struct WalletViewModel: Sendable {
    public let wallet: Wallet

    public init(wallet: Wallet) {
        self.wallet = wallet
    }

    public var name: String {
        wallet.name
    }

    public var subType: String? {
        switch wallet.type {
        case .multicoin:
            return Localized.Wallet.multicoin
        case .view, .single, .privateKey:
            guard let account = wallet.accounts.first else { return .none }
            return GemAddressService.shared.format(address: account.address, chain: account.chain, style: .extra(extra: 1))
        }
    }

    public var image: Image {
        switch wallet.type {
        case .multicoin:
            return Images.Logo.logo
        case .view, .single, .privateKey:
            guard let chain = wallet.accounts.first?.chain else {
                return Images.Logo.logo
            }
            return ChainImage(chain: chain).image
        }
    }

    public var subImage: Image? {
        switch wallet.type {
        case .multicoin, .single, .privateKey: .none
        case .view: Images.Wallets.watch
        }
    }

    public var hasAvatar: Bool {
        wallet.imageUrl != nil
    }

    public var avatarImage: AssetImage {
        AssetImage(
            type: .text(wallet.name),
            imageURL: wallet.imageUrl.map { ImageSource($0).url },
            placeholder: image,
            chainPlaceholder: subImage,
        )
    }
}

extension WalletViewModel: Identifiable, Hashable {
    public var id: String {
        wallet.id.id
    }
}
