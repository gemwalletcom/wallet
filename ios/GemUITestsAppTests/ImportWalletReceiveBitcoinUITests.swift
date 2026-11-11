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

        app.tap("receive_button")
        app.tap("bitcoin")

        XCTAssertTrue(app.element(UITestKitConstants.bitcoinAddress).waitForExistence(timeout: 10))
        app.tapMainAction()

        app.tapBack()
        app.tapBack()
        app.tapWalletBar()

        app.tapImportWallet()

        importFlow(app: app, words: UITestKitConstants.words2)

        app.tapWalletBar()
        XCTAssertEqual(app.getWalletsCount(), 2)
    }

    func importFlow(app: XCUIApplication, words: String) {
        app.tap("multicoin_item")

        app.tap("importInputField")
        app.element("importInputField").typeText(words)
        app.tapMainAction()
        app.skipEnableAuthentication(leaving: "importInputField")
    }
}
