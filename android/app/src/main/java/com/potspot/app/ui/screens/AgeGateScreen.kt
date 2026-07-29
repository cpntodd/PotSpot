package com.potspot.app.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp

@Composable
fun AgeGateScreen(onConfirmed: () -> Unit) {
    Surface(
        modifier = Modifier.fillMaxSize(),
        color = MaterialTheme.colorScheme.background,
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(32.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center,
        ) {
            Text(
                text = "PotSpot",
                style = MaterialTheme.typography.headlineLarge,
                color = MaterialTheme.colorScheme.primary,
            )
            Spacer(modifier = Modifier.height(24.dp))
            Text(
                text = "This app contains information about cannabis strains and is intended for adults only.",
                style = MaterialTheme.typography.bodyLarge,
                textAlign = TextAlign.Center,
            )
            Spacer(modifier = Modifier.height(16.dp))
            Text(
                text = "You must be 18 years of age or older to continue.",
                style = MaterialTheme.typography.titleMedium,
            )
            Spacer(modifier = Modifier.height(32.dp))
            Button(onClick = onConfirmed) {
                Text("I am 18 or older")
            }
            Spacer(modifier = Modifier.height(12.dp))
            TextButton(onClick = { /* exit app - handled by activity */ }) {
                Text("Leave")
            }
        }
    }
}
