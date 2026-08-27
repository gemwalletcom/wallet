// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemNameServiceProtocol
import class Gemstone.GemRecipientService
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

public typealias RecipientDataAction = ((RecipientData) -> Void)?

@Observable
@MainActor
public final class RecipientSceneViewModel {
    public let wallet: Wallet
    public let asset: Asset
    let type: RecipientAssetType

    public let onTransferAction: TransferDataAction

    private let walletSessionService: any WalletSessionManageable
    private let onRecipientDataAction: RecipientDataAction
    private let assetImageFormatter: AssetImageFormatter

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
        walletSessionService: any WalletSessionManageable,
        nameService: any GemNameServiceProtocol,
        type: RecipientAssetType,
        assetImageFormatter: AssetImageFormatter = .shared,
        recipient: RecipientData? = .none,
        onRecipientDataAction: RecipientDataAction,
        onTransferAction: TransferDataAction,
    ) {
        self.wallet = wallet
        self.asset = asset
        self.walletSessionService = walletSessionService
        self.assetImageFormatter = assetImageFormatter
        self.type = type
        self.onRecipientDataAction = onRecipientDataAction
        self.onTransferAction = onTransferAction

        addressInputModel = AddressInputViewModel(
            chain: asset.chain,
            nameService: nameService,
            placeholder: recipientField,
            validators: [
                .required(requireName: recipientField),
                .address(asset),
            ],
        )

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
            imageURL: assetImageFormatter.getNFTUrl(for: nftAsset.id.identifier),
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

    var recipientSections: [ListItemValueSection<Recipient>] {
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

    func onSelectRecipient(_ recipient: Recipient) {
        do {
            let validated = try Recipient(GemRecipientService().recipient(
                chain: asset.chain.rawValue,
                input: recipient.address,
                nameRecord: nil,
                memo: recipient.memo,
                references: [],
            ))
            handle(
                recipientData: RecipientData(
                    recipient: Recipient(name: recipient.name, address: validated.address, memo: validated.memo),
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
    private func sectionRecipients(for section: RecipientAddressType) -> [ListItemValue<Recipient>] {
        switch section {
        case .contacts:
            ContactRecipientSectionViewModel(contacts: contacts).listItems
        case .pinned, .wallets, .view:
            WalletRecipientSectionViewModel(
                wallets: walletSessionService.wallets.filter { $0.id != wallet.id },
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
        switch try Primitives.Payment.decode(string) {
        case let .request(payment):
            try handle(payment: payment)
        case .link:
            throw AnyError(Localized.Errors.invalidAssetAddress(asset.name))
        }
    }

    private func handle(payment: PaymentRequest) throws {
        switch type {
        case let .asset(asset):
            switch try PaymentDestinationBuilder.transfer(payment: payment, asset: asset) {
            case let .confirm(data): handle(transferData: data)
            case let .recipient(data): update(from: data)
            }
        case .nft:
            update(
                from: RecipientData(
                    recipient: Recipient(name: .none, address: chain.checksumAddress(payment.address), memo: payment.memo),
                    amount: .none,
                ),
            )
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
            handle(transferData: TransferData(type: .transferNft(asset), recipient: recipientData.recipient, value: .zero))
        }
    }

    private func handle(transferData: TransferData) {
        onTransferAction?(transferData)
    }
}
