// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemImage
import struct Gemstone.GemRecipient
import BigInt
import protocol Gemstone.GemNameServiceProtocol
import protocol Gemstone.GemRecipientServiceProtocol
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
import struct Gemstone.GemTransferData

public typealias RecipientDataAction = ((RecipientData) -> Void)?

@Observable
@MainActor
public final class RecipientSceneViewModel {
    public let wallet: Wallet
    public let asset: Asset
    let type: RecipientAssetType

    public let onTransferAction: TransferDataAction

    private let service: any GemRecipientServiceProtocol
    private let onRecipientDataAction: RecipientDataAction

    public var isPresentingScanner: RecipientScene.Field?
    var addressInputModel: AddressInputViewModel
    var memo: String = ""
    private(set) var recipientData: RecipientData?

    public let contactsQuery: ObservableQuery<ContactsRequest>
    var contacts: [ContactData] {
        contactsQuery.value
    }

    public init(
        wallet: Wallet,
        asset: Asset,
        service: any GemRecipientServiceProtocol,
        nameService: any GemNameServiceProtocol,
        type: RecipientAssetType,
        recipient: RecipientData? = .none,
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
                recipientData: RecipientData(
                    recipient: try addressInputModel.recipient(memo: memo, references: recipientData?.recipient.references ?? []),
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
                addressInputModel.update(error: error)
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
                recipientData: RecipientData(
                    recipient: GemRecipient(address: validated.address, name: recipient.name, memo: validated.memo),
                    amount: .none,
                ),
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
                wallets: service.recipientWallets(),
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
        switch (try service.scanDestination(url: string, asset: asset.paymentWalletAsset), type) {
        case let (.confirm(transfer), .asset): handle(transferData: service.transferData(transfer: transfer, asset: asset.map()))
        case let (.confirm(transfer), .nft): update(from: RecipientData(recipient: service.transferData(transfer: transfer, asset: asset.map()).recipient, amount: .none))
        case let (.recipient(_, recipient, amount), _): update(from: RecipientData(recipient: recipient, amount: amount))
        case (.selectAsset, _), (.unsupported, _): throw AnyError(Localized.Errors.invalidAssetAddress(asset.name))
        }
    }

    private func update(from recipientData: RecipientData) {
        self.recipientData = recipientData
        addressInputModel.update(text: recipientData.recipient.address)

        if let memo = recipientData.recipient.memo {
            self.memo = memo
        }
    }

    private func handle(recipientData: RecipientData) {
        switch type {
        case .asset:
            onRecipientDataAction?(recipientData)
        case let .nft(asset):
            handle(transferData: GemTransferData(inputType: .transferNft(asset), recipient: recipientData.recipient, value: BigInt.zero))
        }
    }

    private func handle(transferData: GemTransferData) {
        onTransferAction?(transferData)
    }
}
