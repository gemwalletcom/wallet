// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import Style

public struct AddressListItemViewModel {
    public enum AddressStyle {
        case short
        case full
        case extra(Int)

        fileprivate var formatterStyle: AddressFormatter.Style {
            switch self {
            case .short: .short
            case .full: .full
            case let .extra(extra): .extra(extra)
            }
        }
    }

    public enum Mode {
        case auto(addressStyle: AddressStyle)
        case address(addressStyle: AddressStyle)
        case nameOrAddress
    }

    public let title: String
    public let account: SimpleAccount
    public let mode: Mode
    private let addressLink: BlockExplorerLink

    public init(
        title: String,
        account: SimpleAccount,
        mode: Mode,
        addressLink: BlockExplorerLink,
    ) {
        self.title = title
        self.account = account
        self.mode = mode
        self.addressLink = addressLink
    }

    public var subtitle: String {
        switch mode {
        case let .auto(style): auto(for: style)
        case let .address(style): address(for: style)
        case .nameOrAddress: account.name ?? account.address
        }
    }

    public var assetImage: AssetImage? {
        account.assetImage
    }

    public var assetImageStyle: AssetImageView.Style? {
        switch account.addressType {
        case .contact: AssetImageView.Style(foregroundColor: Colors.secondaryText, cornerRadius: 0)
        case .address, .contract, .validator, .internalWallet, .none: nil
        }
    }

    public var assetImageSize: CGFloat {
        switch account.addressType {
        case .contact: .list.accessory
        case .address, .contract, .validator, .internalWallet, .none: .list.image
        }
    }

    public var addressExplorerText: String {
        Localized.Transaction.viewOn(addressLink.name)
    }

    public var addressExplorerUrl: URL {
        addressLink.url
    }

    public var canToggleAddress: Bool {
        guard let name = account.name, name.isNotEmpty else {
            return false
        }
        return name != account.address
    }

    public var addressSubtitle: String {
        address(for: .short)
    }

    // MARK: - Private methods

    private func auto(for style: AddressStyle) -> String {
        if account.name == account.address || account.name == nil {
            return address(for: style)
        } else if let _ = account.assetImage, let name = account.name {
            return name
        } else if let name = account.name {
            let address = address(for: .short)
            if address.isEmpty {
                return name
            }
            return "\(name) (\(address))"
        }
        return account.address
    }

    private func address(for style: AddressStyle) -> String {
        AddressFormatter(style: style.formatterStyle, address: account.address, chain: account.chain).value()
    }
}
