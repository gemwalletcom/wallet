notification_sent_title = 💸 Enviado: {$value}
notification_sent_description = Para {$address}
notification_received_title = 💰 Recibido: {$value}
notification_received_description = De {$address}
notification_unstake_title = 🔒 { notification_action_unstake } {$value}
notification_claim_rewards_title = 🎁 { notification_action_claim_rewards } {$value}
notification_withdraw_title = 🔓 { notification_action_withdraw } {$value}
notification_redelegate_title = 🔄 { notification_action_redelegate } {$value}
notification_swap_title = 🔄 Cambiar de {$from_symbol} a {$to_symbol}
notification_swap_description = {$from_value} > {$to_value}
notification_test = Prueba
notification_token_approval_title = ✅ { notification_action_token_approval } {$token}
notification_stake_title = 🔒 { notification_action_stake } {$value}
notification_price_alert_up_title = 📈 {$symbol} Alerta de precio
notification_price_alert_up_description = El precio aumentó en {$price_change} a {$price}
notification_price_alert_down_title = 📉 {$symbol} Alerta de precio
notification_price_alert_down_description = El precio se redujo en {$price_change} a {$price}
notification_price_alert_all_time_high_title = 🔥 {$symbol} Máximo histórico
notification_price_alert_all_time_high_description = {$symbol} ha alcanzado un nuevo máximo histórico en {$price}.
notification_nft_sent_title = 🖼️ NFT enviado: {$value}
notification_nft_received_title = 🖼️ NFT recibido: {$value}
notification_onboarding_buy_asset_title = 🚀 Comprar {$name}
notification_onboarding_buy_asset_description = Compra {$name} de forma segura hoy: simple, confiable e instantáneo.
notification_fiat_purchase_title = 🚀 Compraste {$value}
notification_fiat_sale_title = 💰 Vendiste {$value}
support_new_message_title = Nuevo mensaje de Soporte
notification_freeze_title = { notification_action_freeze } {$value}
notification_unfreeze_title = { notification_action_unfreeze } {$value}
notification_reward_title = 💎 ¡Ganaste { $value ->
     [one] {$value} punto
    *[other] {$value} puntos
    }!
notification_reward_create_username_description = Usuario listo. Invita a tus amigos para ganar recompensas.
notification_reward_invite_description = El referido se unió usando tu código.
notification_rewards_joined_description = Te uniste usando un código de referencia.
rewards_error_referral_code_not_exist = El código de referencia no existe
rewards_error_referral_device_already_used = Este dispositivo ya se ha utilizado para aplicar un código de referencia
rewards_error_referral_cannot_refer_self = No puedes usar tu propio código de referencia
rewards_error_referral_rewards_not_enabled = Las recompensas no están habilitadas para este usuario
rewards_error_referral_limit_reached = No pudimos verificar tu elegibilidad para la recomendación. Las recompensas por recomendación se otorgan por invitar a amigos que conoces y requieren un dispositivo y una dirección de billetera que no se hayan usado para recomendaciones anteriormente.
rewards_error_referral_referrer_limit_reached = El código de referencia ha alcanzado su límite.
errors_generic = Se produjo un error inesperado. Inténtalo de nuevo más tarde.
fiat_error_limit_reached = Demasiadas solicitudes de cotización. Inténtalo de nuevo en unos minutos.
fiat_error_quote_unavailable = Esta cotización ya no está disponible. Inténtalo de nuevo.
fiat_error_region_unavailable = No disponible en tu región.
rewards_error_referral_country_ineligible = Las referencias no están disponibles actualmente para tu país: {$value}.
notification_rewards_disabled_title = Código de referencia desactivado
notification_rewards_disabled_description = Este código de referencia ha sido deshabilitado debido a repetidas violaciones de los términos de nuestro programa de referencia.
rewards_error_referral_eligibility_expired = Los códigos de referencia solo pueden usarse en un dispositivo y una billetera configurados { $value ->
     [one] en el último día
    *[other] en los últimos {$value} días
    }.
notification_reward_pending_title = 💎 Nueva referencia
rewards_error_username_too_short = El nombre de usuario es demasiado corto. Longitud mínima: {$value}.
rewards_error_username_too_long = El nombre de usuario es demasiado largo. Longitud máxima: {$value}.
rewards_error_username_invalid_characters = El nombre de usuario solo puede contener las letras A–Z y los números 0–9.
rewards_error_username_taken = Este nombre de usuario ya está en uso.
rewards_error_username_wallet_has_username = Esta billetera ya tiene un nombre de usuario.
rewards_error_username_limit_reached = Demasiados intentos de crear un nombre de usuario. Inténtalo de nuevo más tarde.
rewards_error_redemption_not_eligible = Esta cuenta no puede canjear recompensas.
rewards_error_redemption_limit_reached = Has alcanzado el límite de canjes. Inténtalo de nuevo más tarde.
rewards_error_redemption_account_too_new = Tu cuenta es demasiado nueva para canjear recompensas. Inténtalo de nuevo más tarde.
rewards_error_redemption_cooldown = Los canjes se pausan durante un tiempo tras una nueva actividad de referidos. Inténtalo de nuevo más tarde.
rewards_error_redemption_not_enough_points = No tienes suficientes puntos para esta recompensa.
rewards_error_redemption_option_not_available = Esta recompensa ya no está disponible.
rewards_error_redemption_no_username = Crea un nombre de usuario para canjear recompensas.
notification_price_alert_target_title = 🎯 {$symbol} alcanzó {$price}
notification_price_alert_target_description = Ahora en {$price} ({$change}).
notification_stake_rewards_title = 🎁 Recompensas de stake
notification_stake_rewards_description = {$value} recompensas disponibles para {$chain}
notification_rewards_enabled_title = 💎 Recompensas desbloqueadas
notification_rewards_enabled_description = Ya puedes ganar recompensas invitando a tus amigos.
notification_rewards_redeem_points_title = 🎁 Recompensa canjeada
notification_rewards_redeem_points_description = Canjeaste { $value ->
     [one] {$value} punto
    *[other] {$value} puntos
    }.
notification_rewards_redeem_points_for_description = Canjeaste { $points ->
     [one] {$points} punto
    *[other] {$points} puntos
    } por {$value}.
notification_perpetual_long_title = 📈 Largo {$coin}
notification_perpetual_open_description = Entrada a {$price} 🚀
notification_perpetual_short_title = 📉 Corto {$coin}
notification_perpetual_close_positive_description = Ganaste {$pnl} 💰
notification_perpetual_close_negative_description = Perdiste {$pnl} 😕
notification_action_transfer = Transferir
notification_action_swap = Intercambiar
notification_action_token_approval = Aprobar token
notification_action_stake = Stake
notification_action_unstake = Retirar stake
notification_action_redelegate = Redelegar
notification_action_claim_rewards = Reclamar recompensas
notification_action_withdraw = Retirar
notification_action_freeze = Congelar
notification_action_unfreeze = Descongelar
notification_action_smart_contract = Contrato inteligente
notification_transaction_failed_title = ❌ {$action}: Fallido
