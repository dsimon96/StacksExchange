import React from 'react';
import logo from './logo.svg';
import './App.css';

class App extends React.Component {

  componentDidMount() {
    const script = document.createElement("script");
    script.async = true;
    script.defer = true;
    script.src = "https://accounts.google.com/gsi/client";
    document.body.appendChild(script)
  }

  render() {
    return (
      <div className="App">
        <header className="App-header">
          <img src={logo} className="App-logo" alt="logo" />
          <p>
            Edit <code>src/App.tsx</code> and save to reload.
          </p>
          <a
            className="App-link"
            href="https://reactjs.org"
            target="_blank"
            rel="noopener noreferrer"
          >
            Template from Create-React-App
          </a>
          <div className="g_id_signout">Signout implemented here.</div>
        </header>
        <div id="g_id_onload"
          data-client_id="962633347992-tbgvt8rcmnhdp5tlfm2hs1av8bkfc03n.apps.googleusercontent.com"
          data-login_uri="https://localhost:8080/oauth">
        </div>
      </div>
    );
  }
}

export default App;
