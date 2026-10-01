// Copyright (c). Gem Wallet. All rights reserved.

import XCTest

@MainActor
final class ImportWalletReceiveBitcoinUITests: XCTestCase {
    override func setUpWithError() throws {
        try super.setUpWithError()
        continueAfterFailure = false
    }

    func testImportMultiCoinWalletAndVerifyBitcoinAddress() {
        let app = XCUIApplication()
        setupPermissionHandler()
        app.launch()
        app.logout()

        if app.isOnboarding {
            app.tapImportWallet()
        }

        app.acceptTerms()

        importFlow(app: app, words: UITestKitConstants.words)

        app.buttons["receive_button"].firstMatch.tap()

        app.buttons["Bitcoin, BTC"].firstMatch.tap()

        app.buttons["Copy"].firstMatch.tap()
        XCTAssertTrue(app.buttons[UITestKitConstants.bitcoinAddress].exists)

        app.tapBack()
        app.tapBack()
        app.tapWalletBar()

        app.tapImportWallet()

        importFlow(app: app, words: UITestKitConstants.words2)
    }

    func importFlow(app: XCUIApplication, words: String) {
        app.buttons["Multi-Coin"].firstMatch.tap()

        app.textFields["importInputField"].typeText(words)
        app.buttons["Import"].firstMatch.tap()
        app.skipEnableAuthentication()
    }
}
