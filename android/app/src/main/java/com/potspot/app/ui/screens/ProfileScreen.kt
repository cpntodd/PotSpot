package com.potspot.app.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ProfileScreen(onBack: () -> Unit) {
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Profile") },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, "Back")
                    }
                }
            )
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .padding(padding)
                .fillMaxSize()
                .padding(32.dp),
        ) {
            Text("Welcome to PotSpot", style = MaterialTheme.typography.headlineSmall)
            Spacer(Modifier.height(16.dp))
            Text(
                "This is a community-driven cannabis strain catalog. " +
                "Sign in to sync your vault and contribute to the public catalog.",
                style = MaterialTheme.typography.bodyLarge,
            )
            Spacer(Modifier.height(24.dp))
            Button(onClick = { /* TODO: navigate to login */ }) {
                Text("Sign In")
            }
        }
    }
}
