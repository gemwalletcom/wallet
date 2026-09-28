import GemstonePrimitives
import InfoSheet
import Localization
import Style
import SwiftUI

struct JailbreakWarningView: View {
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        InfoSheetScene(
            model: InfoSheetModel(
                title: Localized.Rootcheck.securityAlert,
                description: Localized.Rootcheck.jailbreakBody,
                image: .image(Images.Logo.logo),
                button: .url(AppUrl.docs(.rootedDevice)),
                secondaryButtons: [
                    .action(title: Localized.Common.continueAnyway, action: { dismiss() }),
                ],
            ),
        )
    }
}
