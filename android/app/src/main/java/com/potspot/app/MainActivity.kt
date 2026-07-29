package com.potspot.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.navigation.compose.rememberNavController
import com.potspot.app.ui.navigation.PotSpotNavGraph
import com.potspot.app.ui.theme.PotSpotTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        setContent {
            PotSpotTheme {
                val navController = rememberNavController()
                PotSpotNavGraph(navController = navController)
            }
        }
    }
}
