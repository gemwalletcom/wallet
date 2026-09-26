// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemAvatar
import enum Gemstone.GemConfirmDestination
import enum Gemstone.GemConfirmRowContent
import enum Gemstone.GemListRow
import GemstonePrimitives
import Localization
@testable import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmRowViewModelTests {
    @Test
    func sharedRowsPassThroughAndDetailsDrawNothing() {
        let memo = GemListRow.memo(title: .memo, value: "test memo", menu: [])
        guard case let .row(row) = ConfirmRowViewModel(content: .row(row: memo)).itemModel else {
            Issue.record("Expected a shared row")
            return
        }
        #expect(row == memo)
        #expect(ConfirmRowViewModel(content: .details).isEmpty)
    }

    @Test
    func recipientFollowsTheDestination() throws {
        let cases: [(GemConfirmDestination, String, String)] = [
            (.recipient(name: nil, address: "0xrecipient"), Localized.Transfer.Recipient.title, "0xrecipient"),
            (.contract(name: nil, address: "0xspender"), Localized.Asset.contract, "0xspender"),
            (.validator(name: "Allnodes", address: "validator1"), Localized.Stake.validator, "validator1"),
            (.provider(name: "Yo", address: "0xprovider"), Localized.Common.provider, "0xprovider"),
        ]
        for (destination, title, address) in cases {
            let item = try #require(model(destination, address: address).recipientItem)
            #expect(item.title == title)
            #expect(item.account.address == address)
            #expect(item.onSelect != nil, "a destination with an address can be opened")
        }

        let resource = try #require(model(.resource(resource: Resource.energy.toGem()), address: "").recipientItem)
        #expect(resource.title == Localized.Stake.resource)
        #expect(resource.onSelect == nil, "a resource has no address to open")
    }

    @Test
    func aContactShowsItsPictureOrTheInitialsCoreWrote() throws {
        let withImage = try #require(model(.recipient(name: "Ada", address: "0x1"), address: "0x1", avatar: GemAvatar(imageUrl: "avatar.png", initials: "AD")).recipientItem)
        let withoutImage = try #require(model(.recipient(name: "Ada", address: "0x1"), address: "0x1", avatar: GemAvatar(imageUrl: nil, initials: "AD")).recipientItem)

        #expect(withImage.account.assetImage?.imageURL == ImageSource("avatar.png").url)
        #expect(withoutImage.account.assetImage?.imageURL == nil)
        #expect(withoutImage.account.assetImage?.type == .text("AD"))
    }

    private func model(_ destination: GemConfirmDestination, address: String, avatar: GemAvatar? = nil) -> ConfirmRowViewModel {
        ConfirmRowViewModel(content: .recipient(
            destination: destination,
            name: nil,
            text: address,
            address: address,
            memo: nil,
            chain: Chain.ethereum.rawValue,
            link: BlockExplorerLink.mock().toGem(),
            avatar: avatar,
            isSelectable: !address.isEmpty,
        ), onSelectAddress: { _ in })
    }
}

private extension ConfirmRowViewModel {
    var recipientItem: AddressListItemViewModel? {
        guard case let .recipient(item) = itemModel else { return nil }
        return item
    }

    var isEmpty: Bool {
        if case .empty = itemModel {
            true
        } else {
            false
        }
    }
}
