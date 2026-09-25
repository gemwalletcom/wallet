// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.applicationConnectionRow
import struct Gemstone.GemConnectionRow
import enum Gemstone.GemListRow
import enum Gemstone.GemServiceError
import struct Gemstone.GemSignMessagePreview
import protocol Gemstone.GemSignMessageServiceProtocol
import struct Gemstone.GemSimulationPayloadRow
import struct Gemstone.GemSimulationValue
import struct Gemstone.GemWalletConnectMessageRequest
import func Gemstone.signerFailure
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import WalletConnectorService

@Observable
@MainActor
public final class SignMessageSceneViewModel {
    private let service: any GemSignMessageServiceProtocol
    private let request: GemWalletConnectMessageRequest
    private let confirmTransferDelegate: TransferDataCallback.ConfirmTransferDelegate
    private var preview: GemSignMessagePreview

    public var presentedSheet: SignMessageSheet?
    public var isPresentingAlertMessage: AlertMessage?
    private var hasLoadedAddressNames = false

    public init(
        service: any GemSignMessageServiceProtocol,
        request: GemWalletConnectMessageRequest,
        confirmTransferDelegate: @escaping TransferDataCallback.ConfirmTransferDelegate,
    ) {
        self.service = service
        self.request = request
        self.confirmTransferDelegate = confirmTransferDelegate
        preview = service.preview(request: request)
    }

    private var connection: GemConnectionRow {
        applicationConnectionRow(metadata: request.session.metadata)
    }

    var viewFullMessageListItem: ListItemModel {
        ListItemModel(title: Localized.SignMessage.viewFullMessage)
    }

    var payloadDetailsListItem: ListItemModel {
        ListItemModel(title: Localized.Common.details)
    }

    public var title: String {
        preview.title.text
    }

    public var buttonTitle: String {
        Localized.Transfer.confirm
    }

    var appName: String {
        connection.title
    }

    var rows: [GemListRow] {
        preview.rows
    }

    public var appPreview: AppPreviewModel {
        let connection = connection
        return AppPreviewModel(
            assetImage: AssetImage(imageURL: connection.iconUrl.flatMap(URL.init(string:))),
            name: connection.title,
            subtitleSymbol: connection.host,
        )
    }

    public var headerModel: ValueHeader? {
        headerData?.valueHeader
    }

    public var headerData: GemSimulationValue? {
        preview.header
    }

    var messageText: String {
        preview.text
    }

    public var simulationWarnings: [GemListRow] {
        preview.warnings
    }

    public var primaryPayloadFields: [GemSimulationPayloadRow] {
        preview.primaryFields
    }

    public var secondaryPayloadFields: [GemSimulationPayloadRow] {
        preview.secondaryFields
    }

    public var hasPayloadFields: Bool {
        primaryPayloadFields.isNotEmpty || secondaryPayloadFields.isNotEmpty
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
        let signature = try await service.sign(walletId: request.wallet.id, account: request.account, message: request.message)
        confirmTransferDelegate(.success(signature))
    }

    public func onSign(onComplete: @escaping () -> Void) {
        Task {
            do {
                try await signMessage()
                onComplete()
            } catch let error as GemServiceError {
                switch signerFailure(error: error.text()) {
                case let .retry(text):
                    isPresentingAlertMessage = AlertMessage(title: Localized.Errors.errorOccurred, message: text.text)
                case .reject:
                    confirmTransferDelegate(.failure(error))
                    onComplete()
                }
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

    func fieldModels(for fields: [GemSimulationPayloadRow]) -> [SimulationPayloadFieldViewModel] {
        SimulationPayloadFieldViewModel.models(
            for: fields,
            onSelectAddress: { [weak self] address in
                guard let self else { return }
                presentedSheet = .addressDetails(ChainAddress(chain: Chain(core: request.chain), address: address))
            },
        )
    }

    func onViewPayloadDetails() {
        presentedSheet = .payloadDetails
    }
}

private extension SignMessageSceneViewModel {
    func loadPayloadAddressNamesIfNeeded() async {
        guard !hasLoadedAddressNames, hasPayloadFields else { return }

        hasLoadedAddressNames = true
        preview = await service.withAddressNames(chain: request.chain, preview: preview)
    }
}
