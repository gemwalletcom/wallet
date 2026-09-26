// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemImage
import protocol Gemstone.GemNameServiceProtocol
import struct Gemstone.GemPaymentRecipient
import struct Gemstone.GemRecipient
import enum Gemstone.GemRecipientNext
import protocol Gemstone.GemRecipientServiceProtocol
import struct Gemstone.GemRecipientSession
import enum Gemstone.GemRecipientType
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

@Observable
@MainActor
public final class RecipientSceneViewModel {
    public let wallet: Wallet
    public let asset: Asset
    let type: GemRecipientType

    public let onNavigate: TransferRouteAction

    private let service: any GemRecipientServiceProtocol

    public var isPresentingScanner: RecipientScene.Field?
    var addressInputModel: AddressInputViewModel
    private(set) var session = GemRecipientSession(address: .empty, memo: .empty, payment: nil)

    var memo: String {
        get { session.memo }
        set { session = session.onMemoChanged(memo: newValue) }
    }

    public let contactsQuery: ObservableQuery<ContactsQuery>
    var contacts: [ContactData] {
        contactsQuery.value
    }

    public let walletsQuery = ObservableQuery(WalletsQuery(isPinned: .none), initialValue: [Wallet]())

    public init(
        wallet: Wallet,
        asset: Asset,
        service: any GemRecipientServiceProtocol,
        nameService: any GemNameServiceProtocol,
        type: GemRecipientType,
        recipient: GemPaymentRecipient? = .none,
        onNavigate: TransferRouteAction,
    ) {
        self.wallet = wallet
        self.asset = asset
        self.service = service
        self.type = type
        self.onNavigate = onNavigate

        addressInputModel = AddressInputViewModel(chain: asset.chain, nameService: nameService, placeholder: recipientField)

        contactsQuery = ObservableQuery(ContactsQuery(chain: asset.chain), initialValue: [])

        if let recipient {
            update(from: recipient)
        }
    }

    var tittle: String {
        Localized.Transfer.Recipient.title
    }

    let recipientField = Localized.Transfer.Recipient.addressField
    var memoField: String {
        Localized.Transfer.memo
    }

    func nftAssetImage(for nftAsset: NFTAsset) -> AssetImage {
        AssetImage(
            type: .text("NFT"),
            imageURL: GemImage.nftAsset(assetId: nftAsset.id.identifier).imageURL,
            placeholder: .none,
            chainPlaceholder: .none,
        )
    }

    var actionButtonTitle: String {
        Localized.Common.continue
    }

    var actionButtonState: ButtonState {
        addressInputModel.isValid ? .normal : .disabled
    }

    public var isNextEnabled: Bool {
        actionButtonState == .normal
    }

    var showMemo: Bool {
        asset.chain.isMemoSupported
    }

    var chain: Chain {
        asset.chain
    }

    func listItem(for item: ListItemValue<GemRecipient>) -> ListItemModel {
        ListItemModel(title: item.title ?? item.value.name, subtitle: item.subtitle)
    }

    var recipientSections: [ListItemValueSection<GemRecipient>] {
        service.recipientSections(wallets: walletsQuery.value.map { $0.toGem() }, chain: asset.chain.rawValue, contacts: contacts.map { $0.toGem() })
            .map {
                ListItemValueSection(
                    section: $0.kind.title,
                    image: $0.kind.image,
                    values: $0.rows.map { ListItemValue(title: $0.title, subtitle: $0.subtitle, value: $0.recipient) },
                )
            }
    }

    public func scanType(for field: RecipientScene.Field) -> QRScanType {
        switch field {
        case .address: .address
        case .memo: .memo
        }
    }
}

// MARK: - Actions

extension RecipientSceneViewModel {
    func onContinue() {
        guard addressInputModel.validate() else { return }

        do {
            session = session.onAddressChanged(address: addressInputModel.text)
            try route(session.next(recipientType: type, nameState: addressInputModel.nameResolveState))
        } catch {
            addressInputModel.update(error: error)
        }
    }

    func onSelectScan(field: RecipientScene.Field) {
        isPresentingScanner = field
    }

    public func onHandleScan(_ result: String, for field: RecipientScene.Field) {
        switch field {
        case .address:
            do {
                try scanRecipient(result)
            } catch {
                addressInputModel.update(error: error)
            }

        case .memo:
            memo = result
        }
    }

    func onChangeAddressText(_: String, new: String) {
        session = session.onAddressChanged(address: new)
    }

    func onSelectRecipient(_ recipient: GemRecipient) {
        do {
            try route(service.select(recipientType: type, recipient: recipient))
        } catch {
            addressInputModel.text = recipient.address
            addressInputModel.update(error: error)
        }
    }
}

// MARK: - Private

extension RecipientSceneViewModel {
    private func scanRecipient(_ string: String) throws {
        switch try service.scan(url: string, recipientType: type) {
        case let .confirm(transfer): onNavigate?(.confirm(transfer))
        case let .recipient(payment): update(from: payment)
        }
    }

    private func update(from payment: GemPaymentRecipient) {
        session = session.onPayment(payment: payment)
        addressInputModel.update(text: session.address)
    }

    private func route(_ next: GemRecipientNext) {
        switch next {
        case let .amount(payment): onNavigate?(.amount(AmountInput(type: .transfer(recipient: payment), asset: asset)))
        case let .confirm(transfer): onNavigate?(.confirm(transfer))
        }
    }
}
