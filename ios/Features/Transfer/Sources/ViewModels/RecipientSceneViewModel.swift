// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import GemstonePrimitives
import Keystore
import Localization
import NameService
import NodeService
import Primitives
import PrimitivesComponents
import ScanService
import Store
import Style
import SwiftUI
import WalletSessionService

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
    var amount: String = ""

    public let contactsQuery: ObservableQuery<ContactsRequest>
    var contacts: [ContactData] {
        contactsQuery.value
    }

    public init(
        wallet: Wallet,
        asset: Asset,
        walletSessionService: any WalletSessionManageable,
        nameService: any NameServiceable,
        type: RecipientAssetType,
        assetImageFormatter: AssetImageFormatter = .shared,
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
            type: "NFT",
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

    var recipientSections: [ListItemValueSection<RecipientAddress>] {
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

        handle(
            recipientData: RecipientData(
                recipient: Recipient(
                    name: addressInputModel.nameResolveState.result?.name,
                    address: addressInputModel.resolvedAddress,
                    memo: memo,
                ),
                amount: amount.isEmpty ? .none : amount,
            ),
        )
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

    func onChangeAddressText(_: String, new _: String) {
        if !amount.isEmpty {
            amount = .empty
        }
    }

    func onSelectRecipient(_ recipient: RecipientAddress) {
        handle(
            recipientData: RecipientData(
                recipient: Recipient(
                    name: recipient.name,
                    address: asset.chain.checksumAddress(recipient.address),
                    memo: recipient.memo,
                ),
                amount: .none,
            ),
        )
    }
}

// MARK: - Private

extension RecipientSceneViewModel {
    func getRecipientScanResult(payment: PaymentRequest) throws -> RecipientScanResult {
        let address = asset.chain.checksumAddress(payment.address)
        if let amount = payment.amount, showMemo ? ((payment.memo?.isEmpty) == nil) : true,
           asset.chain.isValidAddress(address)
        {
            let transferType: TransferDataType = switch type {
            case let .asset(asset): .transfer(asset)
            case let .nft(asset): .transferNft(asset)
            }

            let value = try BigNumberFormatter.standard.number(from: amount, decimals: asset.decimals.asInt)
            let recipientData = RecipientData(
                recipient: Recipient(
                    name: .none,
                    address: address,
                    memo: payment.memo,
                ),
                amount: .none,
            )
            return .transferData(
                TransferData(type: transferType, recipientData: recipientData, amount: .exact(value)),
            )
        }

        return .recipient(address: address, memo: payment.memo, amount: payment.amount)
    }

    private func sectionRecipients(for section: RecipientAddressType) -> [ListItemValue<RecipientAddress>] {
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
        let type = try PaymentURLDecoder.decode(string)
        switch type {
        case let .request(payment):
            let scanResult = try getRecipientScanResult(payment: payment)
            switch scanResult {
            case let .transferData(data):
                handle(transferData: data)
            case let .recipient(address, memo, amount):
                // TODO: - open if all fields filled
                addressInputModel.update(text: address)

                if let memo { self.memo = memo }
                if let amount { self.amount = amount }
            }
        case .link: throw AnyError(Localized.Errors.notSupported)
        }
    }

    private func handle(recipientData: RecipientData) {
        switch type {
        case .asset:
            onRecipientDataAction?(recipientData)
        case let .nft(asset):
            handle(transferData: TransferData(type: .transferNft(asset), recipientData: recipientData, amount: .exact(.zero)))
        }
    }

    private func handle(transferData: TransferData) {
        onTransferAction?(transferData)
    }
}
