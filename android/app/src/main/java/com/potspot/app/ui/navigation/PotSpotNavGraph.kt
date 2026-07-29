package com.potspot.app.ui.navigation

import androidx.compose.runtime.Composable
import androidx.navigation.NavHostController
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.navArgument
import com.potspot.app.ui.screens.*

object Routes {
    const val AGE_GATE = "age_gate"
    const val CATALOG = "catalog"
    const val STRAIN_DETAIL = "strain/{strainId}"
    const val VAULT = "vault"
    const val PROFILE = "profile"

    fun strainDetail(strainId: String) = "strain/$strainId"
}

@Composable
fun PotSpotNavGraph(navController: NavHostController) {
    NavHost(navController = navController, startDestination = Routes.AGE_GATE) {
        composable(Routes.AGE_GATE) {
            AgeGateScreen(
                onConfirmed = {
                    navController.navigate(Routes.CATALOG) {
                        popUpTo(Routes.AGE_GATE) { inclusive = true }
                    }
                }
            )
        }
        composable(Routes.CATALOG) {
            CatalogScreen(
                onStrainClick = { strainId ->
                    navController.navigate(Routes.strainDetail(strainId))
                },
                onVaultClick = { navController.navigate(Routes.VAULT) },
                onProfileClick = { navController.navigate(Routes.PROFILE) },
            )
        }
        composable(
            route = Routes.STRAIN_DETAIL,
            arguments = listOf(navArgument("strainId") { type = NavType.StringType }),
        ) { backStackEntry ->
            val strainId = backStackEntry.arguments?.getString("strainId") ?: return@composable
            StrainDetailScreen(strainId = strainId, onBack = { navController.popBackStack() })
        }
        composable(Routes.VAULT) {
            VaultScreen(onBack = { navController.popBackStack() })
        }
        composable(Routes.PROFILE) {
            ProfileScreen(onBack = { navController.popBackStack() })
        }
    }
}
