import React, { useEffect } from 'react';
import logo from './logo.svg';
import { NavigationBar } from './components/Navigation/NavigationBar';

export const App: React.FC = () => {
    useEffect(() => {
      const script = document.createElement("script");
      script.async = true;
      script.defer = true;
      script.src = "https://accounts.google.com/gsi/client";
      document.body.appendChild(script);
    }, [] /* An empty dependencies array means that we will only update one time. */);

    return (
      <div className="App">
        <header>
          <div className="g_id_signout">{/* Signout is implemented here. */}</div>
        </header>
        <div id="g_id_onload"
          data-client_id="962633347992-tbgvt8rcmnhdp5tlfm2hs1av8bkfc03n.apps.googleusercontent.com"
          data-login_uri="http://localhost:8080/oauth">
        </div>

        <NavigationBar />
      </div>
    );
}