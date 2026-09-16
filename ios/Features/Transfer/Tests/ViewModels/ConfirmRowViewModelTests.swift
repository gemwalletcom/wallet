// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemConfirmDestination
import enum Gemstone.GemConfirmRowContent
import func Gemstone.walletRow
import GemstonePrimitives
import Localization
import PrimitivesComponents
@testable import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmRowViewModelTests {
    @Test
    func rowsCarryTheirContent() throws {
        let wallet = Wallet.mock()
        let app = try #require(ConfirmRowViewModel(content: .app(name: "PancakeSwap", iconUrl: nil)).listItem)
        #expect(app.title == Localized.WalletConnect.app)
        #expect(app.subtitle == "PancakeSwap")

        let sender = try #require(ConfirmRowViewModel(content: .sender(wallet: walletRow(wallet: wallet.toGem()))).listItem)
        #expect(sender.title == Localized.Common.wallet)
        #expect(sender.subtitle == wallet.name)
        #expect(sender.imageStyle != nil)

        let network = try #require(ConfirmRowViewModel(content: .network(chain: Chain.ethereum.rawValue, name: "Ethereum (ERC20)")).listItem)
        #expect(network.title == Localized.Transfer.network)
        #expect(network.subtitle == "Ethereum (ERC20)")
        #expect(network.imageStyle != nil)

        let memo = try #require(ConfirmRowViewModel(content: .memo(memo: "test memo")).listItem)
        #expect(memo.title == Localized.Transfer.memo)
        #expect(memo.subtitle == "test memo")
        #expect(try #require(ConfirmRowViewModel(content: .memo(memo: nil)).listItem).subtitle == "-")

        #expect(ConfirmRowViewModel(content: .details).isEmpty)
        #expect(ConfirmRowViewModel(content: nil).isEmpty)
    }

    @Test
    func recipientFollowsTheDestination() throws {
        let cases: [(GemConfirmDestination, String, String)] = [
            (.recipient(name: nil, address: "0xrecipient"), Localized.Transfer.Recipient.title, "0xrecipient"),
            (.contract(address: "0xspender"), Localized.Asset.contract, "0xspender"),
            (.validator(name: "Allnodes", address: "validator1"), Localized.Stake.validator, "validator1"),
            (.provider(name: "Yo", address: "0xprovider"), Localized.Common.provider, "0xprovider"),
        ]
        for (destination, title, address) in cases {
            let item = try #require(model(destination).recipientItem)
            #expect(item.title == title)
            #expect(item.account.address == address)
        }
        #expect(try #require(model(.resource(resource: Resource.energy.toGem())).recipientItem).title == Localized.Stake.resource)
    }

    @Test
    func contactImage() throws {
        let withImage = try #require(model(.recipient(name: nil, address: "0x1"), addressName: .mock(type: .contact, imageUrl: "avatar.png")).recipientItem)
        let withoutImage = try #require(model(.recipient(name: nil, address: "0x1"), addressName: .mock(type: .contact, imageUrl: nil)).recipientItem)
        #expect(withImage.account.assetImage?.imageURL == ImageSource("avatar.png").url)
        #expect(withoutImage.account.assetImage?.imageURL == nil)
    }

    private func model(_ destination: GemConfirmDestination, addressName: AddressName? = nil) -> ConfirmRowViewModel {
        ConfirmRowViewModel(content: .recipient(destination: destination, addressName: addressName?.toGem(), memo: nil, chain: Chain.ethereum.rawValue, link: BlockExplorerLink.mock().toGem()))
    }
}

private extension ConfirmRowViewModel {
    var listItem: ListItemModel? {
        switch itemModel {
        case let .app(item), let .sender(item), let .network(item), let .memo(item): item
        case .header, .recipient, .swapDetails, .networkFee, .perpetualDetails, .perpetualModifyPosition, .warnings, .payload, .balanceChange, .error, .empty: nil
        }
    }

    var recipientItem: AddressListItemViewModel? {
        guard case let .recipient(item) = itemModel else { return nil }
        return item
    }

    var isEmpty: Bool {
        if case .empty = itemModel { true } else { false }
    }
}
