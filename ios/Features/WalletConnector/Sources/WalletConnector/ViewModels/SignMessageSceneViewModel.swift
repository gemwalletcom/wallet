// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemSignMessagePreview
import protocol Gemstone.GemSignMessageServiceProtocol
import Components
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import WalletConnectorService
import struct Gemstone.SimulationPayloadField
import struct Gemstone.GemSimulationWarningRow
import func Gemstone.simulationWarningRows

@Observable
@MainActor
public final class SignMessageSceneViewModel {
    private let service: any GemSignMessageServiceProtocol
    private let payload: SignMessagePayload
    private let confirmTransferDelegate: TransferDataCallback.ConfirmTransferDelegate
    private let preview: GemSignMessagePreview

    public var isPresentingUrl: URL?
    public var isPresentingPayloadDetails: Bool = false
    public var isPresentingAlertMessage: AlertMessage?
    private var payloadAddressNames: [ChainAddress: AddressName] = [:]

    public init(
        service: any GemSignMessageServiceProtocol,
        payload: SignMessagePayload,
        confirmTransferDelegate: @escaping TransferDataCallback.ConfirmTransferDelegate,
    ) {
        self.service = service
        self.payload = payload
        self.confirmTransferDelegate = confirmTransferDelegate
        preview = service.preview(message: payload.message, simulation: payload.simulation, assets: payload.assets.map { $0.map() })
    }

    public var networkText: String {
        payload.chain.networkName
    }

    public var title: String {
        switch preview.messageType {
        case .siwe: Localized.Common.signInWith(Chain.ethereum.networkName)
        case .siws: Localized.Common.signInWith(Chain.solana.networkName)
        case .text, .eip712: Localized.Transfer.reviewRequest
        }
    }

    public var walletText: String {
        payload.wallet.name
    }

    public var buttonTitle: String {
        Localized.Transfer.confirm
    }

    public var connectionViewModel: WalletConnectionViewModel {
        WalletConnectionViewModel(connection: WalletConnection(session: payload.session, wallet: payload.wallet))
    }

    public var appName: String {
        payload.session.metadata.shortName
    }

    public var appAssetImage: AssetImage {
        AssetImage(imageURL: connectionViewModel.imageUrl)
    }

    public var walletAssetImage: AssetImage {
        WalletViewModel(wallet: payload.wallet).avatarImage
    }

    public var networkAssetImage: AssetImage {
        AssetIdViewModel(assetId: payload.chain.asset.id).networkAssetImage
    }

    public var appText: String {
        appName
    }

    public var appPreview: AppPreviewModel {
        AppPreviewModel(
            assetImage: appAssetImage,
            name: appName,
            subtitleSymbol: connectionViewModel.hostText,
        )
    }

    public var headerData: AssetValueHeaderData? {
        preview.header?.map()
    }

    var messageText: String {
        preview.text
    }

    var textMessageViewModel: TextMessageViewModel {
        TextMessageViewModel(message: preview.text)
    }

    public var simulationWarnings: [GemSimulationWarningRow] {
        simulationWarningRows(warnings: payload.simulation.warnings)
    }

    public var payloadModel: SimulationPayloadModel {
        SimulationPayloadModel(
            chain: payload.chain,
            primaryFields: preview.primaryFields,
            secondaryFields: preview.secondaryFields,
            addressNames: payloadAddressNames,
        )
    }

    public var hasWarnings: Bool {
        !simulationWarnings.isEmpty
    }

    public var isButtonDisabled: Bool {
        preview.hasCriticalWarning
    }

    public var buttonType: ButtonType {
        .primary(isButtonDisabled ? .disabled : .normal)
    }

    public func signMessage() async throws {
        let signature = try await service.sign(walletId: payload.wallet.id.id, message: payload.message)
        confirmTransferDelegate(.success(signature))
    }

    public func onSign(onComplete: @escaping () -> Void) {
        Task {
            do {
                try await signMessage()
                onComplete()
            } catch {
                isPresentingAlertMessage = AlertMessage(error: error)
            }
        }
    }
}

// MARK: - Actions

public extension SignMessageSceneViewModel {
    func load() {
        Task {
            await loadPayloadAddressNamesIfNeeded()
        }
    }

    func contextMenuItems(for field: SimulationPayloadField) -> [ContextMenuItemType] {
        payloadModel.contextMenuItems(
            for: field,
            explorerLink: { service.addressUrl(chain: payload.chain.rawValue, address: $0).map() },
            onOpenURL: { [weak self] in self?.isPresentingUrl = $0 },
        )
    }

    func onViewPayloadDetails() {
        isPresentingPayloadDetails = true
    }
}

private extension SignMessageSceneViewModel {
    func loadPayloadAddressNamesIfNeeded() async {
        guard payloadAddressNames.isEmpty, payloadModel.hasFields else { return }

        payloadAddressNames = await service.addressNames(chain: payload.chain, preview: preview)
    }
}
