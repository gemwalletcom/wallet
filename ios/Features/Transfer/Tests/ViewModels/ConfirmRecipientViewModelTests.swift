// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemConfirmDestination
import PrimitivesComponents
import Localization
@testable import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmRecipientViewModelTests {
    @Test
    func rowFollowsTheDestination() throws {
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
        #expect(model(nil).recipientItem == nil)
        #expect(try #require(model(.resource(resource: Resource.energy.json())).recipientItem).title == Localized.Stake.resource)
    }

    @Test
    func addressNameWinsOverRecipientName() throws {
        let item = try #require(model(.recipient(name: "wallet name", address: "0x1"), addressName: .mock(name: "Vitalik.eth")).recipientItem)
        #expect(item.account.name == "Vitalik.eth")
    }

    @Test
    func contactImage() throws {
        let withImage = try #require(model(.recipient(name: nil, address: "0x1"), addressName: .mock(type: .contact, imageUrl: "avatar.png")).recipientItem)
        let withoutImage = try #require(model(.recipient(name: nil, address: "0x1"), addressName: .mock(type: .contact, imageUrl: nil)).recipientItem)
        #expect(withImage.account.assetImage?.imageURL == ImageSource("avatar.png").url)
        #expect(withoutImage.account.assetImage?.imageURL == nil)
    }

    private func model(_ destination: GemConfirmDestination?, addressName: AddressName? = nil) -> ConfirmRecipientViewModel {
        ConfirmRecipientViewModel(destination: destination, chain: .ethereum, memo: nil, addressName: addressName, addressLink: .mock())
    }
}

private extension ConfirmRecipientViewModel {
    var recipientItem: AddressListItemViewModel? {
        guard case let .recipient(item) = itemModel else { return nil }
        return item
    }
}
