import Components
import Foundation
import protocol Gemstone.GemNameServiceProtocol
import enum Gemstone.GemWalletImportKind
import struct Gemstone.GemWalletImportScreen
import struct Gemstone.GemWalletImportSession
import protocol Gemstone.GemWalletServiceProtocol
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

@Observable
@MainActor
final class ImportWalletSceneViewModel {
    private let service: any GemWalletServiceProtocol
    let preferences: ObservablePreferences
    let type: ImportWalletType
    private let importScreen: GemWalletImportScreen

    private(set) var session = GemWalletImportSession(kind: .phrase, text: "", cursor: nil, isImporting: false)
    let nameRecordViewModel: NameRecordViewModel

    var isPresentingScanner = false
    var isPresentingAlertMessage: AlertMessage?
    var isPresentingExistingWalletName: String?

    private let onComplete: VoidAction

    init(
        service: any GemWalletServiceProtocol,
        preferences: ObservablePreferences,
        nameService: any GemNameServiceProtocol,
        type: ImportWalletType,
        onComplete: VoidAction,
    ) {
        self.service = service
        self.preferences = preferences
        self.type = type
        importScreen = switch type {
        case .multicoin: service.importScreen(chain: nil)
        case let .chain(chain): service.importScreen(chain: chain.toGem())
        }
        self.onComplete = onComplete
        nameRecordViewModel = NameRecordViewModel(nameService: nameService)
    }

    var title: String {
        importScreen.title.text
    }

    var input: String {
        get { session.text }
        set { session = session.onInputChanged(text: newValue, cursor: session.cursor) }
    }

    var inputCursor: Int? {
        get { session.cursor.map { Int($0) } }
        set { session = session.onInputChanged(text: session.text, cursor: newValue.map { UInt32($0) }) }
    }

    var importType: GemWalletImportKind {
        get { session.kind }
        set { session = session.onKindChanged(kind: newValue) }
    }

    var showsPhraseSuggestions: Bool {
        importType.supportsPhraseSuggestions() && wordsSuggestion.isNotEmpty
    }

    var wordsSuggestion: [String] {
        session.suggestions()
    }

    var buttonState: ButtonState {
        session.isImporting ? .loading(showProgress: true) : .normal
    }

    var pasteButtonTitle: String {
        Localized.Common.paste
    }

    var pasteButtonImage: Image {
        Images.System.paste
    }

    var qrButtonTitle: String {
        Localized.Wallet.scan
    }

    var qrButtonImage: Image {
        Images.System.qrCodeViewfinder
    }

    var alertTitle: String {
        Localized.Errors.validation("")
    }

    var chain: Chain? {
        switch type {
        case .multicoin: .none
        case let .chain(chain): chain
        }
    }

    var showImportTypes: Bool {
        importScreen.showsKinds
    }

    var importTypes: [GemWalletImportKind] {
        importScreen.kinds
    }

    var footerText: String? {
        importType.showsViewOnlyWarning() ? Localized.Wallet.importAddressWarning : nil
    }

    var docsUrl: URL {
        AppUrl.docs(.howToSecureSecretPhrase)
    }

    var shouldProtectInput: Bool {
        importType.protectsInput()
    }

    var showsNameRecord: Bool {
        importType.resolvesNames()
    }
}

// MARK: - Business Logic

extension ImportWalletSceneViewModel {
    func onChangeInput(_: String, newValue: String) {
        if showsNameRecord, let chain {
            nameRecordViewModel.getNameRecord(name: newValue, chain: chain)
        } else {
            nameRecordViewModel.reset()
        }
    }

    func onSelectActionButton() async {
        session = session.onImporting(isImporting: true)

        do {
            try await importWallet()
        } catch {
            session = session.onImporting(isImporting: false)
            isPresentingAlertMessage = AlertMessage(title: alertTitle, error: error)
        }
    }

    func onSelectScanQR() {
        isPresentingScanner = true
    }

    func onHandleScan(_ result: String) {
        session = session.onInputChanged(text: result, cursor: nil)
    }

    func onSelectWord(_ word: String) {
        session = session.onSuggestionSelected(word: word)
    }

    func onPaste() {
        guard let string = UIPasteboard.general.string else {
            UINotificationFeedbackGenerator().notificationOccurred(.error)
            return
        }
        session = session.onInputChanged(text: string.trim(), cursor: nil)

        if shouldProtectInput {
            CopyTypeViewModel.clearClipboard()
        }
    }

    func onSelectExistingWalletContinue() {
        onComplete?()
    }
}

// MARK: - Private

extension ImportWalletSceneViewModel {
    private func importWallet() async throws {
        let result = try await service.importWallet(
            kind: importType,
            chain: chain,
            input: input,
            nameRecord: showsNameRecord ? nameRecordViewModel.state.record() : nil,
            source: .import,
        )
        session = session.onImporting(isImporting: false)
        switch result {
        case .new: onComplete?()
        case let .existing(wallet): isPresentingExistingWalletName = wallet.name
        }
    }
}
