// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemImage
import struct Gemstone.GemPaymentRecipient
import struct Gemstone.GemRecipient
import protocol Gemstone.GemNameServiceProtocol
import protocol Gemstone.GemRecipientServiceProtocol
import enum Gemstone.GemRecipientType
import Components
import Foundation
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public typealias RecipientDataAction = ((GemPaymentRecipient) -> Void)?

@Observable
@MainActor
public final class RecipientSceneViewModel {
    public let wallet: Wallet
    public let asset: Asset
    let type: GemRecipientType

    public let onTransferAction: TransferDataAction

    private let service: any GemRecipientServiceProtocol
    private let onRecipientDataAction: RecipientDataAction

    public var isPresentingScanner: RecipientScene.Field?
    var addressInputModel: AddressInputViewModel
    var memo: String = ""
    private(set) var recipientData: GemPaymentRecipient?

    public let contactsQuery: ObservableQuery<ContactsRequest>
    var contacts: [ContactData] {
        contactsQuery.value
    }

    public let walletsQuery = ObservableQuery(WalletsRequest(isPinned: .none), initialValue: [Wallet]())

    public init(
        wallet: Wallet,
        asset: Asset,
        service: any GemRecipientServiceProtocol,
        nameService: any GemNameServiceProtocol,
        type: GemRecipientType,
        recipient: GemPaymentRecipient? = .none,
        onRecipientDataAction: RecipientDataAction,
        onTransferAction: TransferDataAction,
    ) {
        self.wallet = wallet
        self.asset = asset
        self.service = service
        self.type = type
        self.onRecipientDataAction = onRecipientDataAction
        self.onTransferAction = onTransferAction

        addressInputModel = AddressInputViewModel(chain: asset.chain, nameService: nameService, placeholder: recipientField)

        contactsQuery = ObservableQuery(ContactsRequest(chain: asset.chain), initialValue: [])

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

    var recipientSections: [ListItemValueSection<GemRecipient>] {
        RecipientAddressType.allCases
            .map {
                ListItemValueSection(
                    section: sectionTitle(for: $0),
                    image: sectionImage(for: $0),
                    values: sectionRecipients(for: $0),
                )
            }
            .filter(\.values.isNotEmpty)
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
            handle(
                recipientData: GemPaymentRecipient(recipient: try addressInputModel.recipient(memo: memo, references: recipientData?.recipient.references ?? []),
                    amount: recipientData?.amount,
                ),
            )
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
                try handleAddressScan(result)
            } catch {
                addressInputModel.update(error: AnyError(Localized.Errors.invalidAssetAddress(asset.name)))
            }

        case .memo:
            memo = result
        }
    }

    func onChangeAddressText(_: String, new: String) {
        guard new != recipientData?.recipient.address else { return }
        recipientData = .none
    }

    func onSelectRecipient(_ recipient: GemRecipient) {
        do {
            let validated = try service.recipient(
                chain: asset.chain.rawValue,
                input: recipient.address,
                nameRecord: nil,
                memo: recipient.memo,
                references: [],
            )
            handle(
                recipientData: GemPaymentRecipient(
                    recipient: GemRecipient(address: validated.address, name: recipient.name, memo: validated.memo)),
            )
        } catch {
            addressInputModel.text = recipient.address
        }
    }
}

// MARK: - Private

extension RecipientSceneViewModel {
    private func sectionRecipients(for section: RecipientAddressType) -> [ListItemValue<GemRecipient>] {
        switch section {
        case .contacts:
            ContactRecipientSectionViewModel(contacts: contacts).listItems
        case .pinned, .wallets, .view:
            WalletRecipientSectionViewModel(
                wallets: service.recipientWallets(wallets: walletsQuery.value),
                section: section,
                chain: asset.chain,
            ).listItems
        }
    }

    private func sectionTitle(for type: RecipientAddressType) -> String {
        switch type {
        case .pinned: Localized.Common.pinned
        case .contacts: Localized.Contacts.title
        case .wallets: Localized.Transfer.Recipient.myWallets
        case .view: Localized.Transfer.Recipient.viewWallets
        }
    }

    private func sectionImage(for type: RecipientAddressType) -> Image {
        switch type {
        case .pinned: Images.System.pin
        case .contacts: Images.System.person
        case .wallets: Images.System.wallet
        case .view: Images.System.eye
        }
    }

    private func handleAddressScan(_ string: String) throws {
        switch try service.scan(url: string, recipientType: type) {
        case let .confirm(transfer): onTransferAction?(transfer)
        case let .recipient(payment): update(from: payment)
        }
    }

    private func update(from recipientData: GemPaymentRecipient) {
        self.recipientData = recipientData
        addressInputModel.update(text: recipientData.recipient.address)

        if let memo = recipientData.recipient.memo {
            self.memo = memo
        }
    }

    private func handle(recipientData: GemPaymentRecipient) {
        switch service.next(recipientType: type, payment: recipientData) {
        case let .amount(payment): onRecipientDataAction?(payment)
        case let .confirm(transfer): onTransferAction?(transfer)
        }
    }
}
